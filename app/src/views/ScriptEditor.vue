<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, nextTick, computed } from 'vue'
import { useScriptStore } from '../stores/script'
import { useMacroStore } from '../stores/macro'
import { useUIStore } from '../stores/ui'
import { useConfigStore } from '../stores/config'
import { Play, Plus, Trash2, Save, Circle, Square, Edit2, Link } from '@lucide/vue'
import CodeEditor from '../components/script/CodeEditor.vue'

const store = useScriptStore()
const macroStore = useMacroStore()
const uiStore = useUIStore()
const configStore = useConfigStore()

const newScriptName = ref('')
const DEFAULT_TEMPLATE = '// 在此编写脚本\n//\n// 指定默认手柄 (首选):\n//   set_default_device(0);\n//\n// 按键操作:\n//   press("A");         - 按下按键\n//   release("A");       - 释放按键\n//\n// 摇杆与扳机:\n//   set_thumb("LeftX", 0.5);    - 设置摇杆 (-1.0 ~ 1.0)\n//   set_trigger("Left", 0.8);   - 设置扳机 (0.0 ~ 1.0)\n//\n// 延时与日志:\n//   sleep(1000);         - 等待毫秒\n//   log("hello");        - 输出日志\n'
const editorCode = ref(DEFAULT_TEMPLATE)

let statusTimer: ReturnType<typeof setTimeout> | null = null

// profile filter
const profileFilter = ref<string>('') // '' = 全部

// scriptId → 绑定的 profile 名称列表
const scriptProfileMap = computed(() => {
  const map: Record<string, string[]> = {}
  for (const profile of configStore.config.profiles) {
    for (const sid of profile.scripts) {
      if (!map[sid]) map[sid] = []
      map[sid].push(profile.name)
    }
  }
  return map
})

// 过滤后的脚本列表
const filteredScripts = computed(() => {
  if (!profileFilter.value) return store.scripts
  if (profileFilter.value === '__unbound__') {
    return store.scripts.filter(s => !scriptProfileMap.value[s.id])
  }
  const profile = configStore.config.profiles.find(p => p.id === profileFilter.value)
  if (!profile) return store.scripts
  return store.scripts.filter(s => profile.scripts.includes(s.id))
})

// 浮动 Tooltip 状态
const tooltipVisible = ref(false)
const tooltipText = ref('')
const tooltipX = ref(0)
const tooltipY = ref(0)

function showTooltip(e: MouseEvent, text: string) {
  tooltipText.value = text
  tooltipX.value = e.clientX + 12
  tooltipY.value = e.clientY - 10
  tooltipVisible.value = true
}

function moveTooltip(e: MouseEvent) {
  tooltipX.value = e.clientX + 12
  tooltipY.value = e.clientY - 10
}

function hideTooltip() {
  tooltipVisible.value = false
}

onMounted(async () => {
  store.fetchScripts()
  configStore.fetchConfig()
})

onUnmounted(() => {
  if (statusTimer) clearTimeout(statusTimer)
})

function clearStatusAfterDelay() {
  if (statusTimer) clearTimeout(statusTimer)
  statusTimer = setTimeout(() => {
    store.executionStatus = 'idle'
    store.executionMessage = ''
  }, 5000)
}

async function startMacroRecording() {
  const name = newScriptName.value.trim() || '录制手柄脚本'
  try {
    await macroStore.startRecord('default', name)
    store.executionStatus = 'running'
    store.executionMessage = '手柄动作捕获中...请操作物理手柄'
  } catch (e) {
    store.executionStatus = 'error'
    store.executionMessage = `录制启动失败: ${e}`
    clearStatusAfterDelay()
  }
}

async function stopMacroRecording() {
  try {
    store.executionStatus = 'running'
    store.executionMessage = '正在停止录制并转换脚本...'
    
    const mac = await macroStore.stopRecord()
    
    // 自动刷新脚本列表
    await store.fetchScripts()
    newScriptName.value = ''
    
    // 自动寻找刚刚生成的转换脚本并选中它
    const targetScriptName = `${mac.name} (自动转换)`
    const newScript = store.scripts.find(s => s.name === targetScriptName)
    if (newScript) {
      await selectScript(newScript.id)
      store.executionStatus = 'success'
      store.executionMessage = `录制并自动转换脚本成功！`
    } else {
      store.executionStatus = 'success'
      store.executionMessage = `录制成功，已自动转换为 Rhai 脚本`
    }
  } catch (e) {
    store.executionStatus = 'error'
    store.executionMessage = `录制终止失败: ${e}`
  } finally {
    clearStatusAfterDelay()
  }
}

watch(() => store.executionStatus, (newVal) => {
  if (newVal === 'success' || newVal === 'error') {
    clearStatusAfterDelay()
  }
})

async function createNewScript() {
  if (!newScriptName.value.trim()) {
    uiStore.showToast('请输入脚本名称后再点击新建', 'warning')
    return
  }
  try {
    const script = await store.createScript(newScriptName.value, DEFAULT_TEMPLATE)
    newScriptName.value = ''
    await selectScript(script.id)
    uiStore.showToast('脚本新建成功', 'success')
  } catch (e) {
    uiStore.showAlert('创建失败', `新建脚本失败: ${e}`)
  }
}

async function selectScript(id: string) {
  await store.getScript(id)
  if (store.currentScript) {
    editorCode.value = store.currentScript.code
  }
}

async function saveScript() {
  if (store.currentScript) {
    try {
      await store.updateScript(store.currentScript.id, editorCode.value)
      uiStore.showToast('脚本保存成功', 'success')
    } catch (e) {
      uiStore.showAlert('保存失败', `保存脚本失败: ${e}`)
    }
  }
}

async function runScript() {
  if (store.currentScript) {
    if (store.executing && store.executionId) {
      await store.stopExecution(store.executionId)
      return
    }
    try {
      await store.executeScript(store.currentScript.id)
    } catch (e) {
      clearStatusAfterDelay()
    }
  }
}

async function deleteScript(id: string) {
  const confirmed = await uiStore.showConfirm('确认删除', '确定要删除这个脚本吗？')
  if (!confirmed) return

  try {
    await store.deleteScript(id)
    uiStore.showToast('脚本删除成功', 'success')
    if (store.currentScript?.id === id) {
      store.currentScript = null
      editorCode.value = DEFAULT_TEMPLATE
    }
  } catch (e) {
    uiStore.showAlert('删除失败', `删除脚本失败: ${e}`)
  }
}

const editingScriptId = ref<string | null>(null)
const editingScriptName = ref('')
const editInput = ref<HTMLInputElement | null>(null)

function startRename(script: any) {
  editingScriptId.value = script.id
  editingScriptName.value = script.name
  nextTick(() => {
    if (editInput.value) {
      editInput.value.focus()
      editInput.value.select()
    }
  })
}

async function saveScriptName(scriptId: string) {
  const name = editingScriptName.value.trim()
  if (!name || name === store.scripts.find(s => s.id === scriptId)?.name) {
    editingScriptId.value = null
    return
  }
  
  try {
    await store.renameScript(scriptId, name)
    uiStore.showToast('重命名成功', 'success')
  } catch (e) {
    uiStore.showAlert('重命名失败', `脚本重命名失败: ${e}`)
  } finally {
    editingScriptId.value = null
  }
}
</script>

<template>
  <div class="script-editor-page" :class="{ 'is-recording': macroStore.isRecording }">
    <div class="page-header">
      <h2>脚本编辑器</h2>
      <div class="header-actions">
        <!-- 捕获录制按钮 -->
        <button
          v-if="!macroStore.isRecording"
          class="btn-record"
          @click="startMacroRecording"
        >
          <Circle :size="14" fill="currentColor" />
          <span>物理手柄宏录制</span>
        </button>
        <button
          v-else
          class="btn-stop-record"
          @click="stopMacroRecording"
        >
          <Square :size="14" fill="currentColor" />
          <span>停止录制</span>
        </button>

        <button class="btn-primary" @click="createNewScript" :disabled="macroStore.isRecording">
          <Plus :size="14" />
          <span>新建</span>
        </button>
        <button class="btn-secondary" @click="saveScript" :disabled="!store.currentScript || macroStore.isRecording">
          <Save :size="14" />
          <span>保存</span>
        </button>
        <button
          class="btn-run"
          :class="{ 'btn-stop': store.executing }"
          @click="runScript"
          :disabled="!store.currentScript || macroStore.isRecording"
        >
          <Play :size="14" />
          <span>{{ store.executing ? '停止' : '运行' }}</span>
        </button>
        <div
          v-if="store.executionStatus !== 'idle'"
          class="execution-status"
          :class="store.executionStatus"
        >
          <span class="status-dot"></span>
          <span class="status-text">{{ store.executionMessage }}</span>
        </div>
      </div>
    </div>

    <div class="editor-layout">
      <div class="script-list-panel">
        <input
          v-model="newScriptName"
          class="input"
          placeholder="脚本名称"
          :disabled="macroStore.isRecording"
        />

        <!-- Profile 分类过滤 -->
        <div class="profile-filter">
          <button
            class="filter-pill"
            :class="{ active: profileFilter === '' }"
            @click="profileFilter = ''"
          >全部</button>
          <button
            v-for="profile in configStore.config.profiles"
            :key="profile.id"
            class="filter-pill"
            :class="{ active: profileFilter === profile.id }"
            @click="profileFilter = profile.id"
            :title="profile.name"
          >{{ profile.name }}</button>
          <button
            class="filter-pill unbound"
            :class="{ active: profileFilter === '__unbound__' }"
            @click="profileFilter = '__unbound__'"
          >未绑定</button>
        </div>

        <div class="script-list">
          <div v-if="filteredScripts.length === 0" class="list-empty">无匹配脚本</div>
          <div
            v-for="script in filteredScripts"
            :key="script.id"
            class="script-item"
            :class="{ active: store.currentScript?.id === script.id, disabled: macroStore.isRecording, bound: !!scriptProfileMap[script.id] }"
            @click="!macroStore.isRecording && selectScript(script.id)"
          >
            <template v-if="editingScriptId === script.id">
              <input
                ref="editInput"
                v-model="editingScriptName"
                class="edit-name-input"
                @keydown.enter="saveScriptName(script.id)"
                @blur="saveScriptName(script.id)"
                @click.stop
              />
            </template>
            <template v-else>
              <div class="script-name-row">
                <span class="script-name" @dblclick="!macroStore.isRecording && startRename(script)">{{ script.name }}</span>
                <!-- 已绑定徽章 + 浮动 Tooltip -->
                <span
                  v-if="scriptProfileMap[script.id]"
                  class="bound-badge"
                  @mouseenter="showTooltip($event, '绑定于: ' + scriptProfileMap[script.id].join(', '))"
                  @mousemove="moveTooltip"
                  @mouseleave="hideTooltip"
                >
                  <Link :size="9" />
                </span>
              </div>
              <div class="script-item-actions">
                <button class="icon-btn" @click.stop="!macroStore.isRecording && startRename(script)" :disabled="macroStore.isRecording" title="重命名">
                  <Edit2 :size="12" />
                </button>
                <button class="icon-btn danger" @click.stop="!macroStore.isRecording && deleteScript(script.id)" :disabled="macroStore.isRecording" title="删除">
                  <Trash2 :size="12" />
                </button>
              </div>
            </template>
          </div>
        </div>
      </div>

      <div class="editor-panel">
        <CodeEditor v-model="editorCode" :activeLine="store.activeLine" />
      </div>

      <div class="api-panel">
        <h4>API参考</h4>
        <div class="api-section">
          <h5>指定默认手柄 (首选)</h5>
          <code>set_default_device(0);</code>
          <code>// 在脚本最上方指定默认手柄后</code>
          <code>// 下面所有函数均可省略手柄编号！</code>
        </div>
        <div class="api-section">
          <h5>按键操作</h5>
          <code>press("A"); // 默认手柄</code>
          <code>press(0, "A"); // 指定手柄</code>
          <code>release("A");</code>
          <code>按键: A B X Y LB RB LT RT</code>
          <code>BACK START GUIDE LS RS</code>
          <code>UP DOWN LEFT RIGHT</code>
        </div>
        <div class="api-section">
          <h5>摇杆 (-1.0 ~ 1.0)</h5>
          <code>set_thumb(axis, val);</code>
          <code>set_thumb(0, axis, val);</code>
          <code>set_thumb("LeftX", 1.0);</code>
          <code>axis: LeftX LeftY RightX RightY</code>
        </div>
        <div class="api-section">
          <h5>扳机 (0.0 ~ 1.0)</h5>
          <code>set_trigger(side, val);</code>
          <code>set_trigger(0, side, val);</code>
          <code>set_trigger("Left", 0.5);</code>
          <code>side: Left Right</code>
        </div>
        <div class="api-section">
          <h5>OCR 屏幕文本识别</h5>
          <code>ocr() // 默认标定区 #1 识别</code>
          <code>ocr(index) // 读取标定区序号并识别</code>
          <code>ocr(x, y, w, h) // 指定屏幕区域识别</code>
          <code>// 亚像素 ClearType 级别高清对齐</code>
          <code>// 深色模式智能自适应反色</code>
          <code>// 自动过滤空格/换行，方便字符判定</code>
        </div>
        <div class="api-section">
          <h5>字符串模糊判断与匹配</h5>
          <code>let text = ocr(1);</code>
          <code>text.contains("确定") // 模糊匹配</code>
          <code>text.is_empty() // 是否为空字串</code>
          <code>text == "开始游戏" // 精确相等比较</code>
          <code>text.len // 获取识别字数 (属性)</code>
          <code>log("结果: " + text); // 拼接输出</code>
        </div>
        <div class="api-section">
          <h5>延时与日志</h5>
          <code>sleep(ms);</code>
          <code>log("message");</code>
        </div>
        <div class="api-section">
          <h5>变量与运算</h5>
          <code>let x = 10;</code>
          <code>let name = "hello";</code>
          <code>let flag = true;</code>
          <code>+ - * / % 比较运算</code>
          <code>== != &lt; &gt; &lt;= &gt;=</code>
        </div>
        <div class="api-section">
          <h5>条件判断</h5>
          <code>if x &gt; 5 { ... }</code>
          <code>if x &gt; 5 { ... }</code>
          <code>else { ... }</code>
          <code>if x == 1 { ... }</code>
          <code>else if x == 2 { ... }</code>
        </div>
        <div class="api-section">
          <h5>循环</h5>
          <code>while flag { ... }</code>
          <code>loop { ... break; }</code>
          <code>for i in 0..10 { ... }</code>
          <code>break / continue</code>
        </div>
        <div class="api-section">
          <h5>函数</h5>
          <code>fn add(a, b) {</code>
          <code>  return a + b;</code>
          <code>}</code>
          <code>add(1, 2)</code>
        </div>
        <div class="api-section">
          <h5>数组与对象</h5>
          <code>let arr = [1, 2, 3];</code>
          <code>arr[0] // 访问</code>
          <code>arr.push(4);</code>
          <code>let obj = #{a: 1};</code>
          <code>obj.a // 访问</code>
        </div>
      </div>
    </div>

    <!-- 浮动 Tooltip 层，挂在根元素内，不受 overflow:hidden 影响 -->
    <teleport to="body">
      <div
        v-if="tooltipVisible"
        class="floating-tooltip"
        :style="{ left: tooltipX + 'px', top: tooltipY + 'px' }"
      >{{ tooltipText }}</div>
    </teleport>
  </div>
</template>

<style scoped>
.script-editor-page {
  padding: var(--space-lg);
  height: 100%;
  display: flex;
  flex-direction: column;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--space-md);
}

.page-header h2 {
  font-size: 18px;
  font-weight: 600;
}

.header-actions {
  display: flex;
  gap: var(--space-sm);
}

.btn-primary,
.btn-secondary,
.btn-run {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  padding: var(--space-sm) var(--space-md);
  border-radius: var(--radius-md);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-primary {
  background: var(--color-cta);
  color: white;
}

.btn-primary:hover {
  background: var(--color-cta-hover);
}

.btn-secondary {
  background: var(--color-surface-elevated);
  color: var(--color-text);
  border: 1px solid var(--color-border);
}

.btn-secondary:hover {
  border-color: var(--color-cta);
}

.btn-run {
  background: var(--color-info);
  color: white;
}

.btn-run:hover {
  opacity: 0.9;
}

.btn-run.btn-stop {
  background: var(--color-error);
}

.execution-status {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: var(--space-sm) var(--space-md);
  border-radius: var(--radius-md);
  font-size: 12px;
  font-weight: 500;
  animation: fadeIn 0.3s ease;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(-4px); }
  to { opacity: 1; transform: translateY(0); }
}

.execution-status.running {
  background: rgba(59, 130, 246, 0.12);
  color: #60a5fa;
}

.execution-status.success {
  background: rgba(34, 197, 94, 0.12);
  color: #4ade80;
}

.execution-status.error {
  background: rgba(239, 68, 68, 0.12);
  color: #f87171;
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
}

.execution-status.running .status-dot {
  background: #60a5fa;
  animation: pulse 1.5s ease-in-out infinite;
}

.execution-status.success .status-dot {
  background: #4ade80;
}

.execution-status.error .status-dot {
  background: #f87171;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.3; }
}

.status-text {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 240px;
}

.btn-primary:disabled,
.btn-secondary:disabled,
.btn-run:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.editor-layout {
  flex: 1;
  display: flex;
  gap: var(--space-md);
  min-height: 0;
}

.script-list-panel {
  width: 210px;
  min-width: 210px;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  padding: var(--space-sm);
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
  overflow: hidden;
}

.input {
  padding: var(--space-sm) var(--space-md);
  background: var(--color-surface-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  color: var(--color-text);
  font-size: 12px;
  flex-shrink: 0;
}

.input:focus {
  border-color: var(--color-cta);
}

/* Profile 分类过滤 */
.profile-filter {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  flex-shrink: 0;
}

.filter-pill {
  font-size: 10px;
  font-weight: 500;
  padding: 2px 7px;
  border-radius: 10px;
  cursor: pointer;
  border: 1px solid var(--color-border);
  background: var(--color-surface-elevated);
  color: var(--color-text-dim);
  transition: all var(--transition-fast);
  white-space: nowrap;
  max-width: 80px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.filter-pill:hover {
  color: var(--color-text);
  border-color: var(--color-text-dim);
}

.filter-pill.active {
  background: rgba(99, 102, 241, 0.15);
  border-color: var(--color-cta);
  color: var(--color-cta);
}

.filter-pill.unbound.active {
  background: rgba(245, 158, 11, 0.12);
  border-color: #f59e0b;
  color: #f59e0b;
}

.list-empty {
  font-size: 11px;
  color: var(--color-text-dim);
  text-align: center;
  padding: var(--space-md);
}

.script-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
  overflow-y: auto;
  flex: 1;
  min-height: 0;
}

.script-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--space-xs) var(--space-sm);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: background var(--transition-fast);
  position: relative;
}

.script-item-actions {
  display: flex;
  align-items: center;
  gap: 2px;
  flex-shrink: 0;
}

.edit-name-input {
  background: var(--color-surface-elevated);
  border: 1px solid var(--color-cta);
  color: var(--color-text);
  font-size: 12px;
  padding: 2px 6px;
  border-radius: var(--radius-sm);
  width: 100%;
  outline: none;
  font-family: inherit;
}

.script-item:hover {
  background: var(--color-surface-elevated);
}

.script-item.active {
  background: rgba(34, 197, 94, 0.1);
  color: var(--color-cta);
}

.script-name-row {
  display: flex;
  align-items: center;
  gap: 4px;
  flex: 1;
  min-width: 0;
  overflow: hidden;
}

.script-name {
  font-size: 12px;
  color: var(--color-text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
  min-width: 0;
}

.script-item.active .script-name {
  color: var(--color-cta);
}

/* 已绑定徽章 + Tooltip */
.bound-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: rgba(99, 102, 241, 0.2);
  color: var(--color-cta);
  flex-shrink: 0;
  position: relative;
  cursor: default;
}

.icon-btn {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-sm);
  color: var(--color-text-dim);
  cursor: pointer;
  opacity: 0;
  transition: all var(--transition-fast);
}

.script-item:hover .icon-btn {
  opacity: 1;
}

.icon-btn.danger:hover {
  background: rgba(239, 68, 68, 0.15);
  color: var(--color-error);
}

.editor-panel {
  flex: 1;
  min-width: 0;
}

.code-editor {
  width: 100%;
  height: 100%;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  padding: var(--space-md);
  color: var(--color-text);
  font-family: var(--font-heading);
  font-size: 13px;
  line-height: 1.6;
  resize: none;
  tab-size: 2;
}

.code-editor:focus {
  border-color: var(--color-cta);
}

.api-panel {
  width: 260px;
  min-width: 260px;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  padding: var(--space-md);
  overflow-y: auto;
}

.api-panel h4 {
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text);
  margin-bottom: var(--space-md);
}

.api-section {
  margin-bottom: var(--space-md);
}

.api-section h5 {
  font-size: 11px;
  color: var(--color-cta);
  margin-bottom: var(--space-xs);
}

.api-section code {
  display: block;
  font-size: 11px;
  color: var(--color-text-muted);
  font-family: var(--font-heading);
  padding: 2px 0;
}

.btn-record {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  padding: var(--space-sm) var(--space-md);
  border-radius: var(--radius-md);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
  background: var(--color-error);
  color: white;
}

.btn-record:hover {
  opacity: 0.9;
  transform: translateY(-0.5px);
}

.btn-stop-record {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  padding: var(--space-sm) var(--space-md);
  border-radius: var(--radius-md);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
  background: var(--color-warning);
  color: var(--color-primary);
  animation: pulse-recording 1.5s infinite;
}

.btn-stop-record:hover {
  opacity: 0.9;
}

@keyframes pulse-recording {
  0% {
    box-shadow: 0 0 0 0 rgba(239, 68, 68, 0.4);
  }
  70% {
    box-shadow: 0 0 0 6px rgba(239, 68, 68, 0);
  }
  100% {
    box-shadow: 0 0 0 0 rgba(239, 68, 68, 0);
  }
}

.script-item.disabled {
  opacity: 0.5;
  cursor: not-allowed !important;
}

.script-editor-page.is-recording .editor-panel {
  opacity: 0.6;
  pointer-events: none;
}

.script-editor-page.is-recording .api-panel {
  opacity: 0.6;
  pointer-events: none;
}
</style>

<!-- 全局样式：teleport 层不受 scoped 应用，必须单独一个非 scoped 块 -->
<style>
.floating-tooltip {
  position: fixed;
  z-index: 99999;
  pointer-events: none;
  background: #1e2130;
  border: 1px solid rgba(255, 255, 255, 0.12);
  color: #e2e8f0;
  font-size: 11px;
  font-family: 'Inter', 'Outfit', system-ui, sans-serif;
  padding: 5px 10px;
  border-radius: 6px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
  max-width: 260px;
  word-break: break-all;
  line-height: 1.5;
  animation: tooltipFadeIn 0.12s ease-out;
}

@keyframes tooltipFadeIn {
  from { opacity: 0; transform: translateY(2px); }
  to   { opacity: 1; transform: translateY(0); }
}
</style>
