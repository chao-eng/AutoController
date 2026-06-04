<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, nextTick, computed, defineAsyncComponent } from 'vue'
import { useScriptStore } from '../stores/script'
import { useMacroStore } from '../stores/macro'
import { useUIStore } from '../stores/ui'
import { useConfigStore } from '../stores/config'
import { Play, Plus, Trash2, Save, Circle, Square, Edit2, Link, BookOpen, ChevronLeft, ChevronRight, List, PanelLeftClose, PanelLeftOpen } from '@lucide/vue'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import PageShell from '@/components/layout/PageShell.vue'
import PageHeader from '@/components/layout/PageHeader.vue'

const CodeEditor = defineAsyncComponent(() => import('../components/script/CodeEditor.vue'))

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

// API参考栏折叠状态
let collapsedDefault = false
try {
  collapsedDefault = localStorage.getItem('api_panel_collapsed') === 'true'
} catch (e) {
  console.warn('Unable to access localStorage:', e)
}
const apiPanelCollapsed = ref(collapsedDefault)

let scriptListCollapsedDefault = false
try {
  scriptListCollapsedDefault = localStorage.getItem('script_list_collapsed') === 'true'
} catch (e) {
  console.warn('Unable to access localStorage:', e)
}
const scriptListCollapsed = ref(scriptListCollapsedDefault)

function toggleScriptList() {
  scriptListCollapsed.value = !scriptListCollapsed.value
  try {
    localStorage.setItem('script_list_collapsed', String(scriptListCollapsed.value))
  } catch (e) {
    console.warn('Unable to write to localStorage:', e)
  }
}

function handleScriptListClick() {
  if (scriptListCollapsed.value) {
    toggleScriptList()
  }
}

function toggleApiPanel() {
  apiPanelCollapsed.value = !apiPanelCollapsed.value
  try {
    localStorage.setItem('api_panel_collapsed', String(apiPanelCollapsed.value))
  } catch (e) {
    console.warn('Unable to write to localStorage:', e)
  }
}

function handleApiPanelClick() {
  if (apiPanelCollapsed.value) {
    toggleApiPanel()
  }
}

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

function handleKeyDown(e: KeyboardEvent) {
  if ((e.ctrlKey || e.metaKey) && e.key === 's') {
    e.preventDefault()
    saveScript()
  }
}

onMounted(async () => {
  store.fetchScripts()
  configStore.fetchConfig()
  window.addEventListener('keydown', handleKeyDown)
})

onUnmounted(() => {
  if (statusTimer) clearTimeout(statusTimer)
  window.removeEventListener('keydown', handleKeyDown)
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

// ── 拖拽与排序控制 ──────────────────────────────────────────
const draggedIndex = ref<number | null>(null)
const dragOverIndex = ref<number | null>(null)

function handleDragStart(index: number, event: DragEvent) {
  if (macroStore.isRecording) return
  draggedIndex.value = index
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = 'move'
    event.dataTransfer.setData('text/plain', index.toString())
  }
}

function handleDragEnter(index: number, event: DragEvent) {
  dragOverIndex.value = index
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = 'move'
  }
}

function handleDragOver(_index: number, event: DragEvent) {
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = 'move'
  }
}

function handleDragLeave() {
  dragOverIndex.value = null
}

async function handleDrop(targetIndex: number) {
  dragOverIndex.value = null
  if (draggedIndex.value === null || draggedIndex.value === targetIndex || macroStore.isRecording) return
  
  const orderedList = [...filteredScripts.value]
  const [removed] = orderedList.splice(draggedIndex.value, 1)
  orderedList.splice(targetIndex, 0, removed)
  
  const otherScripts = store.scripts.filter(s => !orderedList.find(o => o.id === s.id))
  store.scripts = [...orderedList, ...otherScripts]
  
  saveScriptOrder(store.scripts)
  draggedIndex.value = null
}



function saveScriptOrder(orderedScripts: any[]) {
  const ids = orderedScripts.map(s => s.id)
  localStorage.setItem('script_order', JSON.stringify(ids))
}

</script>

<template>
  <PageShell :scroll="false" :class="macroStore.isRecording ? 'is-recording' : ''">
    <PageHeader
      title="脚本编辑器"
      description="编写、录制、运行和管理 Rhai 自动化脚本。"
    >
      <template #actions>
        <Button
          v-if="!macroStore.isRecording"
          variant="destructive"
          size="sm"
          @click="startMacroRecording"
        >
          <Circle :size="14" fill="currentColor" />
          <span>物理手柄宏录制</span>
        </Button>
        <Button
          v-else
          size="sm"
          class="animate-pulse-recording border-orange-500/40 bg-orange-500/15 text-orange-600 hover:bg-orange-500/25 dark:text-orange-400"
          @click="stopMacroRecording"
        >
          <Square :size="14" fill="currentColor" />
          <span>停止录制</span>
        </Button>

        <Button variant="default" size="sm" @click="createNewScript" :disabled="macroStore.isRecording">
          <Plus :size="14" />
          <span>新建</span>
        </Button>
        <Button variant="outline" size="sm" @click="saveScript" :disabled="!store.currentScript || macroStore.isRecording">
          <Save :size="14" />
          <span>保存</span>
        </Button>
        <Button
          :variant="store.executing ? 'destructive' : 'default'"
          size="sm"
          @click="runScript"
          :disabled="!store.currentScript || macroStore.isRecording"
        >
          <Play :size="14" />
          <span>{{ store.executing ? '停止' : '运行' }}</span>
        </Button>
        <div
          v-if="store.executionStatus !== 'idle'"
          :class="[
            'flex items-center gap-1.5 rounded-md px-3 py-1.5 text-xs font-medium animate-fadeIn shrink-0',
            store.executionStatus === 'running' ? 'bg-blue-500/12 text-blue-400' : '',
            store.executionStatus === 'success' ? 'bg-green-500/12 text-green-400' : '',
            store.executionStatus === 'error' ? 'bg-red-500/12 text-red-400' : '',
          ]"
        >
          <span
            :class="[
              'h-1.5 w-1.5 shrink-0 rounded-full',
              store.executionStatus === 'running' ? 'bg-blue-400 animate-pulse' : '',
              store.executionStatus === 'success' ? 'bg-green-400' : '',
              store.executionStatus === 'error' ? 'bg-red-400' : '',
            ]"
          ></span>
          <span class="max-w-[240px] truncate whitespace-nowrap">{{ store.executionMessage }}</span>
        </div>
      </template>
    </PageHeader>

    <div class="flex min-h-0 flex-1 gap-4">
      <div
        :class="[
          'relative flex flex-col overflow-hidden rounded-lg border border-border bg-surface transition-all duration-300',
          scriptListCollapsed ? 'w-12 min-w-12 cursor-pointer p-1 hover:border-primary hover:bg-surface-elevated' : 'w-[260px] min-w-[260px] gap-2 p-2'
        ]"
        @click="handleScriptListClick"
      >
        <div v-if="scriptListCollapsed" class="flex h-full min-h-0 flex-col items-center gap-2">
          <button
            class="flex h-6 w-6 items-center justify-center rounded-sm text-muted-foreground/70 opacity-80 transition-all hover:bg-white/8 hover:text-text hover:opacity-100"
            @click.stop="toggleScriptList"
            title="展开脚本列表"
          >
            <PanelLeftOpen :size="14" />
          </button>
          <span class="whitespace-nowrap text-[11px] font-semibold uppercase tracking-[4px] text-muted-foreground/60" style="writing-mode: vertical-lr">脚本</span>
          <div class="flex min-h-0 flex-1 flex-col items-center gap-1 overflow-y-auto">
            <button
              v-for="script in filteredScripts"
              :key="script.id"
              class="script-rail-button"
              :class="{ active: store.currentScript?.id === script.id }"
              :title="script.name"
              :disabled="macroStore.isRecording"
              @click.stop="!macroStore.isRecording && selectScript(script.id)"
            >
              {{ script.name.trim().slice(0, 1) || '#' }}
            </button>
            <span v-if="filteredScripts.length === 0" class="pt-2 text-[11px] text-muted-foreground/50">—</span>
          </div>
        </div>

        <template v-else>
          <div class="flex shrink-0 items-center justify-between">
            <div class="flex min-w-0 items-center gap-1 text-text">
              <List :size="14" />
              <h4 class="m-0 truncate text-xs font-semibold text-text">脚本列表</h4>
              <span class="rounded bg-muted px-1.5 py-0.5 text-[10px] font-semibold text-muted-foreground">{{ filteredScripts.length }}</span>
            </div>
            <button
              class="flex h-5 w-5 items-center justify-center rounded-sm text-muted-foreground/70 opacity-70 transition-all hover:opacity-100 hover:bg-white/8 hover:text-text"
              @click.stop="toggleScriptList"
              title="折叠为一列"
            >
              <PanelLeftClose :size="14" />
            </button>
          </div>

          <Input
            v-model="newScriptName"
            placeholder="脚本名称"
            :disabled="macroStore.isRecording"
            class="shrink-0 text-xs"
          />

          <div class="flex shrink-0 flex-wrap gap-1">
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

          <div class="flex min-h-0 flex-1 flex-col gap-0.5 overflow-y-auto" @dragover.prevent>
            <div v-if="filteredScripts.length === 0" class="py-4 text-center text-[11px] text-muted-foreground/60">无匹配脚本</div>
            <div
              v-for="(script, index) in filteredScripts"
              :key="script.id"
              :class="[
                'flex items-center justify-between rounded px-2 py-1 transition-colors select-none',
                store.currentScript?.id === script.id ? 'bg-green-500/10 text-green-500' : '',
                macroStore.isRecording ? 'cursor-not-allowed opacity-50' : 'cursor-pointer',
                dragOverIndex === index ? 'border border-dashed border-primary bg-primary/5' : '',
              ]"
              draggable="true"
              @dragstart="handleDragStart(index, $event)"
              @dragover.prevent="handleDragOver(index, $event)"
              @dragenter.prevent="handleDragEnter(index, $event)"
              @dragleave="handleDragLeave"
              @drop.prevent="handleDrop(index)"
              @click="!macroStore.isRecording && selectScript(script.id)"
            >
              <template v-if="editingScriptId === script.id">
                <input
                  ref="editInput"
                  v-model="editingScriptName"
                  class="w-full rounded-sm border border-primary bg-surface-elevated px-1.5 py-0.5 text-xs text-text outline-none"
                  @keydown.enter="saveScriptName(script.id)"
                  @blur="saveScriptName(script.id)"
                  @click.stop
                />
              </template>
              <template v-else>
                <div class="flex min-w-0 flex-1 items-center gap-1 overflow-hidden" draggable="false">
                  <span class="cursor-grab pr-1 text-[13px] text-muted-foreground/60 select-none active:cursor-grabbing" title="按住拖拽排序" draggable="false">☰</span>
                  <span
                    class="min-w-0 flex-1 truncate text-xs text-muted-foreground"
                    :class="store.currentScript?.id === script.id ? 'text-green-500' : ''"
                    @dblclick="!macroStore.isRecording && startRename(script)"
                    draggable="false"
                  >{{ script.name }}</span>
                  <span
                    v-if="scriptProfileMap[script.id]"
                    class="inline-flex h-3.5 w-3.5 shrink-0 cursor-default items-center justify-center rounded-full bg-indigo-500/20 text-primary"
                    @mouseenter="showTooltip($event, '绑定于: ' + scriptProfileMap[script.id].join(', '))"
                    @mousemove="moveTooltip"
                    @mouseleave="hideTooltip"
                    draggable="false"
                  >
                    <Link :size="9" />
                  </span>
                </div>
                <div class="flex items-center gap-0.5 shrink-0">
                  <Button variant="ghost" size="icon-xs" :disabled="macroStore.isRecording" title="重命名" @click.stop="!macroStore.isRecording && startRename(script)">
                    <Edit2 :size="12" />
                  </Button>
                  <Button variant="ghost" size="icon-xs" class="hover:text-destructive hover:bg-destructive/15" :disabled="macroStore.isRecording" title="删除" @click.stop="!macroStore.isRecording && deleteScript(script.id)">
                    <Trash2 :size="12" />
                  </Button>
                </div>
              </template>
            </div>
          </div>
        </template>
      </div>

      <div class="min-w-0 flex-1">
        <CodeEditor v-model="editorCode" :activeLine="store.activeLine" @save="saveScript" />
      </div>

      <div
        :class="[
          'relative flex flex-col overflow-y-auto rounded-lg border border-border bg-surface p-4 transition-all duration-300',
          apiPanelCollapsed ? 'w-12 min-w-12 cursor-pointer overflow-hidden px-1 hover:border-primary hover:bg-surface-elevated' : 'w-[260px] min-w-[260px]'
        ]"
        @click="handleApiPanelClick"
      >
        <div :class="['flex shrink-0 select-none items-center justify-between', apiPanelCollapsed ? 'mb-0 h-full flex-col gap-4 justify-start' : 'mb-4']">
          <div :class="['flex items-center gap-1 text-text', apiPanelCollapsed ? 'mt-1 flex-col' : '']">
            <BookOpen :size="14" />
            <h4 v-if="!apiPanelCollapsed" class="m-0 text-xs font-semibold text-text">API参考</h4>
            <span v-else class="mt-2 whitespace-nowrap text-[11px] font-semibold uppercase tracking-[4px] text-muted-foreground/60" style="writing-mode: vertical-lr">API参考</span>
          </div>
          <button
            class="flex h-5 w-5 items-center justify-center rounded-sm text-muted-foreground/70 opacity-70 transition-all hover:opacity-100 hover:bg-white/8 hover:text-text"
            @click.stop="toggleApiPanel"
            :title="apiPanelCollapsed ? '展开API参考' : '折叠API参考'"
          >
            <ChevronRight :size="14" v-if="!apiPanelCollapsed" />
            <ChevronLeft :size="14" v-else />
          </button>
        </div>
        <div v-show="!apiPanelCollapsed" class="flex flex-1 flex-col overflow-y-auto">
          <div class="mb-4">
            <h5 class="mb-1 text-[11px] text-primary">指定默认手柄 (首选)</h5>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">set_default_device(0);</code>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">// 在脚本最上方指定默认手柄后</code>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">// 下面所有函数均可省略手柄编号！</code>
          </div>
          <div class="mb-4">
            <h5 class="mb-1 text-[11px] text-primary">按键操作</h5>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">press("A"); // 默认手柄</code>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">press(0, "A"); // 指定手柄</code>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">release("A");</code>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">按键: A B X Y LB RB LT RT</code>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">BACK START GUIDE LS RS</code>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">UP DOWN LEFT RIGHT</code>
          </div>
          <div class="mb-4">
            <h5 class="mb-1 text-[11px] text-primary">摇杆 (-1.0 ~ 1.0)</h5>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">set_thumb(axis, val);</code>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">set_thumb(0, axis, val);</code>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">set_thumb("LeftX", 1.0);</code>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">axis: LeftX LeftY RightX RightY</code>
          </div>
          <div class="mb-4">
            <h5 class="mb-1 text-[11px] text-primary">扳机 (0.0 ~ 1.0)</h5>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">set_trigger(side, val);</code>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">set_trigger(0, side, val);</code>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">set_trigger("Left", 0.5);</code>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">side: Left Right</code>
          </div>
          <div class="mb-4">
            <h5 class="mb-1 text-[11px] text-primary">OCR 屏幕文本识别</h5>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">ocr() // 默认标定区 #1 识别</code>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">ocr(index) // 读取标定区序号并识别</code>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">ocr(x, y, w, h) // 指定屏幕区域识别</code>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">// 亚像素 ClearType 级别高清对齐</code>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">// 深色模式智能自适应反色</code>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">// 自动过滤空格/换行，方便字符判定</code>
          </div>
          <div class="mb-4">
            <h5 class="mb-1 text-[11px] text-primary">字符串模糊判断与匹配</h5>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">let text = ocr(1);</code>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">text.contains("确定") // 模糊匹配</code>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">text.is_empty() // 是否为空字串</code>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">text == "开始游戏" // 精确相等比较</code>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">text.len // 获取识别字数 (属性)</code>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">text.to_int() // 字符串转为整数</code>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">log("结果: " + text); // 拼接输出</code>
          </div>
          <div class="mb-4">
            <h5 class="mb-1 text-[11px] text-primary">延时与日志</h5>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">sleep(ms);</code>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">log("message");</code>
          </div>
          <div class="mb-4">
            <h5 class="mb-1 text-[11px] text-primary">变量与运算</h5>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">let x = 10;</code>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">let name = "hello";</code>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">let flag = true;</code>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">+ - * / % 比较运算</code>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">== != &lt; &gt; &lt;= &gt;=</code>
          </div>
          <div class="mb-4">
            <h5 class="mb-1 text-[11px] text-primary">条件判断</h5>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">if x &gt; 5 { ... }</code>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">else { ... }</code>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">if x == 1 { ... }</code>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">else if x == 2 { ... }</code>
          </div>
          <div class="mb-4">
            <h5 class="mb-1 text-[11px] text-primary">循环</h5>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">while flag { ... }</code>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">loop { ... break; }</code>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">for i in 0..10 { ... }</code>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">break / continue</code>
          </div>
          <div class="mb-4">
            <h5 class="mb-1 text-[11px] text-primary">函数</h5>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">fn add(a, b) {</code>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">  return a + b;</code>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">}</code>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">add(1, 2)</code>
          </div>
          <div class="mb-4">
            <h5 class="mb-1 text-[11px] text-primary">数组与对象</h5>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">let arr = [1, 2, 3];</code>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">arr[0] // 访问</code>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">arr.push(4);</code>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">let obj = #{a: 1};</code>
            <code class="block py-0.5 font-mono text-[11px] text-muted-foreground">obj.a // 访问</code>
          </div>
        </div>
      </div>
    </div>

    <teleport to="body">
      <div
        v-if="tooltipVisible"
        class="floating-tooltip"
        :style="{ left: tooltipX + 'px', top: tooltipY + 'px' }"
      >{{ tooltipText }}</div>
    </teleport>
  </PageShell>
</template>

<style scoped>
.filter-pill {
  font-size: 10px;
  font-weight: 500;
  padding: 2px 7px;
  border-radius: 10px;
  cursor: pointer;
  border: 1px solid var(--border);
  background: var(--color-surface-elevated);
  color: var(--muted-foreground);
  opacity: 0.6;
  transition: all 0.15s ease;
  white-space: nowrap;
  max-width: 80px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.filter-pill:hover {
  opacity: 1;
  color: var(--foreground);
  border-color: var(--muted-foreground);
}

.filter-pill.active {
  opacity: 1;
  background: rgba(99, 102, 241, 0.15);
  border-color: var(--primary);
  color: var(--primary);
}

.filter-pill.unbound.active {
  background: rgba(245, 158, 11, 0.12);
  border-color: #f59e0b;
  color: #f59e0b;
}

.script-rail-button {
  width: 1.7rem;
  height: 1.7rem;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  border: 1px solid var(--border);
  border-radius: 7px;
  background: var(--color-surface-elevated);
  color: var(--muted-foreground);
  font-size: 11px;
  font-weight: 800;
  line-height: 1;
  transition: all 0.15s ease;
}

.script-rail-button:hover {
  border-color: var(--primary);
  color: var(--foreground);
}

.script-rail-button.active {
  border-color: var(--primary);
  background: rgba(99, 102, 241, 0.15);
  color: var(--primary);
}

.is-recording .min-w-0.flex-1:first-of-type,
.is-recording [class*="w-\\[260px\\]"]:last-of-type {
  opacity: 0.6;
  pointer-events: none;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(-4px); }
  to { opacity: 1; transform: translateY(0); }
}

.animate-fadeIn {
  animation: fadeIn 0.3s ease;
}

@keyframes pulse-recording {
  0% { box-shadow: 0 0 0 0 rgba(239, 68, 68, 0.4); }
  70% { box-shadow: 0 0 0 6px rgba(239, 68, 68, 0); }
  100% { box-shadow: 0 0 0 0 rgba(239, 68, 68, 0); }
}

.animate-pulse-recording {
  animation: pulse-recording 1.5s infinite;
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
