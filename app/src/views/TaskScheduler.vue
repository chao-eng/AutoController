<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useSchedulerStore } from '../stores/scheduler'
import { useScriptStore } from '../stores/script'
import { useUIStore } from '../stores/ui'
import { useConfigStore } from '../stores/config'
import { Plus, ToggleLeft, ToggleRight, Trash2, Play, Square, ArrowUp, ArrowDown, X } from '@lucide/vue'
import type { ScheduledTask } from '../types/scheduler'

const store = useSchedulerStore()
const scriptStore = useScriptStore()
const uiStore = useUIStore()
const configStore = useConfigStore()

// 新建/编辑任务序列状态
const showEditor = ref(false)
const taskName = ref('')
const taskLoopCount = ref(1)
const steps = ref<{ script_id: string; loop_count: number }[]>([])
const selectedNotificationChannels = ref<string[]>([])

// 调度配置表单状态
const scheduleType = ref<'once' | 'daily' | 'interval' | 'cron' | 'manual'>('once')
const onceDateTime = ref(new Date(Date.now() + 60000).toISOString().substring(0, 16))
const dailyTime = ref('12:00:00')
const intervalDuration = ref(5)
const intervalUnit = ref<'seconds' | 'minutes' | 'hours'>('minutes')
const cronExpression = ref('*/5 * * * *')
const taskPriority = ref(1)

onMounted(async () => {
  store.fetchTasks()
  scriptStore.fetchScripts()
  configStore.fetchConfig()
})

function getTypeName(type: string): string {
  switch (type) {
    case 'feishu': return '飞书'
    case 'serverchan': return 'Server酱'
    case 'serverchan3': return 'Server酱³'
    case 'telegram': return 'Telegram'
    default: return type
  }
}

function openEditor() {
  taskName.value = ''
  taskLoopCount.value = 1
  steps.value = []
  selectedNotificationChannels.value = []
  scheduleType.value = 'manual'
  onceDateTime.value = new Date(Date.now() + 60000).toISOString().substring(0, 16)
  dailyTime.value = '12:00:00'
  intervalDuration.value = 5
  intervalUnit.value = 'minutes'
  cronExpression.value = '*/5 * * * *'
  taskPriority.value = 1
  if (scriptStore.scripts.length > 0) {
    // 默认添加一个步骤方便操作
    steps.value.push({
      script_id: scriptStore.scripts[0].id,
      loop_count: 1,
    })
  }
  showEditor.value = true
}


function addStep() {
  if (scriptStore.scripts.length > 0) {
    steps.value.push({
      script_id: scriptStore.scripts[0].id,
      loop_count: 1,
    })
  } else {
    uiStore.showAlert('提示', '请先在“脚本”页面创建一些脚本后再添加步骤！')
  }
}

function removeStep(index: number) {
  steps.value.splice(index, 1)
}

function moveStepUp(index: number) {
  if (index > 0) {
    const temp = steps.value[index]
    steps.value[index] = steps.value[index - 1]
    steps.value[index - 1] = temp
  }
}

function moveStepDown(index: number) {
  if (index < steps.value.length - 1) {
    const temp = steps.value[index]
    steps.value[index] = steps.value[index + 1]
    steps.value[index + 1] = temp
  }
}

function getScriptName(scriptId: string): string {
  const s = scriptStore.scripts.find((s) => s.id === scriptId)
  return s ? s.name : '未知脚本'
}

async function saveTask() {
  if (!taskName.value.trim()) {
    uiStore.showToast('请输入任务序列名称', 'warning')
    return
  }
  if (steps.value.length === 0) {
    uiStore.showToast('请至少添加一个脚本步骤', 'warning')
    return
  }

  // 根据选定类型动态构造 schedule 对象
  let schedule: any;
  if (scheduleType.value === 'once') {
    schedule = { once: new Date(onceDateTime.value).toISOString() };
  } else if (scheduleType.value === 'daily') {
    let time = dailyTime.value;
    if (time.split(':').length === 2) {
      time = `${time}:00`;
    }
    schedule = { daily: { time } };
  } else if (scheduleType.value === 'interval') {
    let ms = intervalDuration.value * 1000;
    if (intervalUnit.value === 'minutes') {
      ms *= 60;
    } else if (intervalUnit.value === 'hours') {
      ms *= 3600;
    }
    schedule = { interval: { duration_ms: ms } };
  } else if (scheduleType.value === 'cron') {
    schedule = { cron: { expression: cronExpression.value } };
  } else {
    schedule = 'manual'; // 手动执行类型直接对应 Rust 序列化后的 'manual' 字符串
  }

  const newTask: ScheduledTask = {
    id: uuidv4(),
    name: taskName.value,
    schedule,
    action: {
      execute_sequence: {
        steps: steps.value.map((s) => ({
          script_id: s.script_id,
          loop_count: s.loop_count,
        })),
        task_loop_count: taskLoopCount.value,
      },
    },
    priority: taskPriority.value,
    enabled: true,
    last_run: null,
    next_run: null,
    notification_channels: [...selectedNotificationChannels.value],
  }

  try {
    await store.createTask(newTask)
    showEditor.value = false
    taskName.value = ''
    taskLoopCount.value = 1
    steps.value = []
    selectedNotificationChannels.value = []
    uiStore.showToast('新建任务序列与调度成功', 'success')
  } catch (e) {
    uiStore.showAlert('创建失败', `创建任务序列失败: ${e}`)
  }

}

function uuidv4() {
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function (c) {
    const r = (Math.random() * 16) | 0,
      v = c == 'x' ? r : (r & 0x3) | 0x8
    return v.toString(16)
  })
}

function getScheduleLabel(schedule: any): string {
  if (schedule === 'manual') return '🖱️ 手动执行 (不自动触发)';
  if (schedule.once) {
    const date = new Date(schedule.once);
    return `单次定时: ${date.toLocaleString()}`;
  }
  if (schedule.daily) return `每日定时: ${schedule.daily.time}`;
  if (schedule.interval) {
    const ms = schedule.interval.duration_ms;
    if (ms >= 3600000) return `循环间隔: ${ms / 3600000} 小时`;
    if (ms >= 60000) return `循环间隔: ${ms / 60000} 分钟`;
    return `循环间隔: ${ms / 1000} 秒`;
  }
  if (schedule.cron) return `Cron: ${schedule.cron.expression}`;
  return '未知';
}

function getActionLabel(action: any): string {
  if (action.play_macro) return `回放宏`
  if (action.execute_script) return `执行单脚本: ${getScriptName(action.execute_script.script_id)}`
  if (action.execute_sequence) {
    const count = action.execute_sequence.steps.length
    return `多脚本串联序列 (步骤数: ${count})`
  }
  return '未知'
}

function getSequenceSummary(action: any): string {
  if (!action.execute_sequence) return ''
  return action.execute_sequence.steps
    .map((s: any) => `${getScriptName(s.script_id)} (${s.loop_count}次)`)
    .join(' → ')
}

function hasSequence(action: any): boolean {
  return !!action.execute_sequence
}

function getSequenceLoopCount(action: any): number {
  return action.execute_sequence?.task_loop_count || 1
}

async function startSequence(taskId: string) {
  try {
    await store.executeSequence(taskId)
  } catch (e) {
    console.error('运行序列失败:', e)
  }
}

async function stopSequence(taskId: string) {
  await store.stopSequence(taskId)
}
</script>

<template>
  <div class="task-scheduler">
    <div class="page-header">
      <h2>任务调度与序列控制</h2>
      <button class="btn-primary" @click="openEditor" :disabled="store.executingSequence">
        <Plus :size="14" />
        <span>新建任务序列</span>
      </button>
    </div>

    <!-- 正在运行的序列全局状态面板 -->
    <div v-if="store.executingSequence && store.sequenceProgress" class="running-progress-panel">
      <div class="progress-panel-header">
        <div class="progress-title">
          <span class="pulse-dot"></span>
          <h4>串联任务序列正在运行...</h4>
        </div>
        <button class="btn-stop-global" @click="stopSequence(store.executingTaskId!)">
          <Square :size="12" fill="currentColor" />
          <span>强制终止序列</span>
        </button>
      </div>

      <div class="progress-grid">
        <!-- 整体循环进度 -->
        <div class="progress-card">
          <div class="card-meta">
            <span>序列总体循环</span>
            <span class="value-highlight">{{ store.sequenceProgress.current_task_loop }} / {{ store.sequenceProgress.total_task_loops }} 轮</span>
          </div>
          <div class="progress-bar-container">
            <div
              class="progress-bar-fill task-fill"
              :style="{ width: `${(store.sequenceProgress.current_task_loop / store.sequenceProgress.total_task_loops) * 100}%` }"
            ></div>
          </div>
        </div>

        <!-- 当前脚本的内部循环进度 -->
        <div class="progress-card">
          <div class="card-meta">
            <span>当前脚本: <strong class="script-highlight">{{ store.sequenceProgress.current_script_name }}</strong></span>
            <span class="value-highlight">{{ store.sequenceProgress.current_step_loop }} / {{ store.sequenceProgress.total_step_loops }} 次</span>
          </div>
          <div class="progress-bar-container">
            <div
              class="progress-bar-fill step-fill"
              :style="{ width: `${(store.sequenceProgress.current_step_loop / store.sequenceProgress.total_step_loops) * 100}%` }"
            ></div>
          </div>
        </div>
      </div>

      <div class="progress-footer">
        <span class="footer-step">
          正在执行第 {{ store.sequenceProgress.current_step_index + 1 }} / {{ store.sequenceProgress.total_steps }} 个脚本步骤
        </span>
      </div>
    </div>

    <div class="task-list">
      <div v-if="store.tasks.length === 0" class="empty-state">
        暂无串联任务，点击右上角"新建任务序列"开始设计编排吧
      </div>

      <div
        v-for="task in store.tasks"
        :key="task.id"
        class="task-item"
        :class="{ disabled: !task.enabled, active: store.executingTaskId === task.id }"
      >
        <div class="task-info">
          <div class="name-row">
            <span class="task-name">{{ task.name }}</span>
            <span v-if="hasSequence(task.action)" class="badge-sequence">序列</span>
          </div>
          <span class="task-schedule">{{ getScheduleLabel(task.schedule) }}</span>
          <span class="task-action">{{ getActionLabel(task.action) }}</span>
          
          <!-- 如果是序列，展示序列链概览 -->
          <div v-if="hasSequence(task.action)" class="sequence-chain-box">
            <span class="sequence-chain-title">串联链条:</span>
            <p class="sequence-chain-text">{{ getSequenceSummary(task.action) }}</p>
          </div>
        </div>

        <div class="task-meta">
          <span class="task-priority">轮数: {{ getSequenceLoopCount(task.action) }}</span>
        </div>

        <div class="task-actions">
          <!-- 运行 / 停止控制 -->
          <button
            v-if="store.executingTaskId !== task.id"
            class="action-play-btn"
            @click="startSequence(task.id)"
            :disabled="store.executingSequence || !task.enabled"
            title="运行串联序列"
          >
            <Play :size="14" fill="currentColor" />
          </button>
          <button
            v-else
            class="action-stop-btn"
            @click="stopSequence(task.id)"
            title="终止运行"
          >
            <Square :size="14" fill="currentColor" />
          </button>

          <!-- 启用禁用 -->
          <button
            class="icon-btn"
            @click="store.toggleTask(task.id, !task.enabled)"
            :disabled="store.executingTaskId === task.id"
            :title="task.enabled ? '禁用' : '启用'"
          >
            <ToggleRight v-if="task.enabled" :size="18" color="var(--color-cta)" />
            <ToggleLeft v-else :size="18" />
          </button>

          <!-- 删除 -->
          <button
            class="icon-btn danger"
            @click="store.removeTask(task.id)"
            :disabled="store.executingTaskId === task.id"
            title="删除"
          >
            <Trash2 :size="14" />
          </button>
        </div>
      </div>
    </div>

    <!-- 新建任务编排弹窗 -->
    <Transition name="fade">
      <div v-if="showEditor" class="editor-backdrop" @click.self="showEditor = false">
        <div class="editor-modal">
          <div class="modal-header">
            <h3>编排串联多脚本序列</h3>
            <button class="close-btn" @click="showEditor = false">
              <X :size="18" />
            </button>
          </div>

          <div class="modal-body">
            <div class="form-group">
              <label>序列名称</label>
              <input v-model="taskName" class="input" placeholder="输入任务序列名称 (例如: 刷圈结算一体化)" />
            </div>

            <div class="form-group">
              <label>整个任务的执行轮数 (Loop Count)</label>
              <input v-model.number="taskLoopCount" type="number" min="1" max="999" class="input small" />
            </div>

            <!-- 任务调度策略配置 -->
            <div class="scheduler-config-box">
              <h4 class="scheduler-config-title">⏰ 任务调度配置</h4>
              
              <div class="form-group">
                <label>调度类型 (Schedule Type)</label>
                <select v-model="scheduleType" class="select-input">
                  <option value="manual">🖱️ 手动执行 (Manual) — 不自动触发</option>
                  <option value="once">📅 单次定时执行 (Once)</option>
                  <option value="daily">🕒 每日固定时间 (Daily)</option>
                  <option value="interval">🔁 周期循环间隔 (Interval)</option>
                  <option value="cron">⚡ 标准 Cron 表达式 (Cron)</option>
                </select>
              </div>

              <!-- Manual (手动执行) -->
              <div v-if="scheduleType === 'manual'" class="manual-info-box fade-in">
                <span class="manual-info-icon">🖱️</span>
                <div class="manual-info-text">
                  <strong>手动执行模式</strong>
                  <p>系统调度器不会自动触发此任务。仅在你点击任务卡片上的 <strong>▶ 运行</strong> 按钮时执行一次。</p>
                </div>
              </div>

              <!-- Once (单次执行) -->
              <div v-if="scheduleType === 'once'" class="form-group fade-in">
                <label>执行时间</label>
                <input v-model="onceDateTime" type="datetime-local" class="input" />
              </div>

              <!-- Daily (每日定时) -->
              <div v-if="scheduleType === 'daily'" class="form-group fade-in">
                <label>每日固定时间 (时:分:秒)</label>
                <input v-model="dailyTime" type="time" step="1" class="input" />
              </div>

              <!-- Interval (周期循环) -->
              <div v-if="scheduleType === 'interval'" class="form-group row-group fade-in">
                <div class="field-item">
                  <label>执行间隔</label>
                  <input v-model.number="intervalDuration" type="number" min="1" class="input" />
                </div>
                <div class="field-item">
                  <label>时间单位</label>
                  <select v-model="intervalUnit" class="select-input">
                    <option value="seconds">秒 (Seconds)</option>
                    <option value="minutes">分钟 (Minutes)</option>
                    <option value="hours">小时 (Hours)</option>
                  </select>
                </div>
              </div>

              <!-- Cron (Cron 表达式) -->
              <div v-if="scheduleType === 'cron'" class="form-group fade-in">
                <label>标准 Cron 表达式 (5字段: 分 时 日 月 周)</label>
                <input v-model="cronExpression" class="input" placeholder="*/5 * * * * (每 5 分钟)" />
                <span class="input-hint">例如: <code>0 12 * * *</code> (每日中午 12 点), <code>*/30 * * * *</code> (每半小时)</span>
              </div>

              <!-- 任务优先级 (手动模式下隐藏) -->
              <div v-if="scheduleType !== 'manual'" class="form-group">
                <label>调度抢占优先级 (1-100，数字越大优先级越高)</label>
                <input v-model.number="taskPriority" type="number" min="1" max="100" class="input small" />
              </div>
            </div>

            <!-- 聚合通知配置 -->
            <div class="scheduler-config-box">
              <h4 class="scheduler-config-title">🔔 任务通知配置</h4>
              <div class="form-group">
                <label>选择通知通道 (任务完成或中断时发送通知)</label>
                <div v-if="!configStore.config.notification_channels || configStore.config.notification_channels.length === 0" class="notify-empty-hint">
                  暂无配置好的通知通道。你可以先去“通知配置”页面添加。
                </div>
                <div v-else class="notify-checkbox-list">
                  <label v-for="ch in configStore.config.notification_channels" :key="ch.id" class="notify-checkbox-item">
                    <input 
                      type="checkbox" 
                      :value="ch.id" 
                      v-model="selectedNotificationChannels" 
                    />
                    <span class="notify-ch-name">{{ ch.name }}</span>
                    <span class="notify-ch-type">({{ getTypeName(ch.config.type) }})</span>
                  </label>
                </div>
              </div>
            </div>

            <div class="steps-container">

              <div class="steps-header">
                <h4>串联执行步骤顺序 (顺序从上到下)</h4>
                <button class="btn-add-step" @click="addStep">
                  <Plus :size="12" />
                  <span>添加串联步骤</span>
                </button>
              </div>

              <div class="steps-list">
                <div v-if="steps.length === 0" class="steps-empty">
                  点击“添加串联步骤”绑定脚本
                </div>
                <div v-for="(step, index) in steps" :key="index" class="step-editor-item">
                  <div class="step-index">#{{ index + 1 }}</div>
                  
                  <!-- 脚本下拉选择 -->
                  <div class="step-select">
                    <select v-model="step.script_id" class="select-input">
                      <option v-for="s in scriptStore.scripts" :key="s.id" :value="s.id">
                        {{ s.name }}
                      </option>
                    </select>
                  </div>

                  <!-- 重复次数 -->
                  <div class="step-loop">
                    <input v-model.number="step.loop_count" type="number" min="1" max="999" class="loop-input" title="单步循环次数" />
                    <span class="loop-label">次</span>
                  </div>

                  <!-- 排序/移除操作 -->
                  <div class="step-actions">
                    <button class="sort-btn" @click="moveStepUp(index)" :disabled="index === 0" title="上移">
                      <ArrowUp :size="12" />
                    </button>
                    <button class="sort-btn" @click="moveStepDown(index)" :disabled="index === steps.length - 1" title="下移">
                      <ArrowDown :size="12" />
                    </button>
                    <button class="remove-btn" @click="removeStep(index)" title="移除">
                      <Trash2 :size="12" />
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <div class="modal-footer">
            <button class="btn-cancel" @click="showEditor = false">取消</button>
            <button class="btn-save" @click="saveTask">保存序列</button>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.task-scheduler {
  padding: var(--space-lg);
  height: 100%;
  overflow-y: auto;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--space-lg);
}

.page-header h2 {
  font-size: 18px;
  font-weight: 600;
}

.btn-primary {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  background: var(--color-cta);
  color: white;
  padding: var(--space-sm) var(--space-md);
  border-radius: var(--radius-md);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-primary:hover:not(:disabled) {
  background: var(--color-cta-hover);
  transform: translateY(-1px);
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* 正在运行进度条面板 */
.running-progress-panel {
  background: linear-gradient(135deg, rgba(34, 197, 94, 0.08) 0%, rgba(20, 83, 45, 0.05) 100%);
  border: 1px solid rgba(34, 197, 94, 0.25);
  border-radius: var(--radius-lg);
  padding: var(--space-md) var(--space-lg);
  margin-bottom: var(--space-lg);
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
  animation: slideIn 0.3s ease;
}

@keyframes slideIn {
  from { opacity: 0; transform: translateY(-10px); }
  to { opacity: 1; transform: translateY(0); }
}

.progress-panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--space-md);
}

.progress-title {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}

.progress-title h4 {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-cta);
  margin: 0;
}

.pulse-dot {
  width: 8px;
  height: 8px;
  background-color: var(--color-cta);
  border-radius: 50%;
  animation: pulse-dot 1.5s infinite;
}

@keyframes pulse-dot {
  0% { transform: scale(0.9); opacity: 1; }
  50% { transform: scale(1.4); opacity: 0.4; }
  100% { transform: scale(0.9); opacity: 1; }
}

.btn-stop-global {
  display: flex;
  align-items: center;
  gap: 6px;
  background: var(--color-error);
  color: white;
  padding: 6px 12px;
  border-radius: var(--radius-sm);
  font-size: 11px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-stop-global:hover {
  opacity: 0.9;
}

.progress-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--space-lg);
  margin-bottom: var(--space-sm);
}

@media (max-width: 768px) {
  .progress-grid {
    grid-template-columns: 1fr;
    gap: var(--space-md);
  }
}

.progress-card {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: var(--space-sm) var(--space-md);
}

.card-meta {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 12px;
  color: var(--color-text-dim);
  margin-bottom: var(--space-xs);
}

.script-highlight {
  color: var(--color-text);
  font-weight: 600;
}

.value-highlight {
  font-family: var(--font-heading);
  font-weight: 600;
  color: var(--color-cta);
}

.progress-bar-container {
  height: 6px;
  background: var(--color-surface-elevated);
  border-radius: 3px;
  overflow: hidden;
}

.progress-bar-fill {
  height: 100%;
  border-radius: 3px;
  transition: width 0.3s ease;
}

.progress-bar-fill.task-fill {
  background: linear-gradient(90deg, #10b981 0%, #059669 100%);
}

.progress-bar-fill.step-fill {
  background: linear-gradient(90deg, #3b82f6 0%, #2563eb 100%);
}

.progress-footer {
  font-size: 11px;
  color: var(--color-text-dim);
  text-align: right;
  font-family: var(--font-heading);
}

.task-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
}

.empty-state {
  text-align: center;
  color: var(--color-text-dim);
  padding: var(--space-2xl);
  font-size: 13px;
}

.task-item {
  display: flex;
  align-items: center;
  gap: var(--space-md);
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  padding: var(--space-md) var(--space-lg);
  transition: all var(--transition-normal);
}

.task-item:hover {
  border-color: var(--color-cta);
  box-shadow: 0 4px 12px rgba(51, 112, 255, 0.05);
}

.task-item.active {
  border-color: var(--color-cta);
  background: linear-gradient(135deg, rgba(51, 112, 255, 0.03) 0%, rgba(30, 58, 138, 0.01) 100%);
}

.task-item.disabled {
  opacity: 0.5;
}

.task-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.name-row {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}

.task-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text);
}

.badge-sequence {
  font-size: 10px;
  font-weight: 600;
  background: rgba(51, 112, 255, 0.15);
  color: var(--color-cta);
  padding: 1px 6px;
  border-radius: 4px;
}

.task-schedule,
.task-action {
  font-size: 11px;
  color: var(--color-text-dim);
  font-family: var(--font-heading);
}

.sequence-chain-box {
  margin-top: var(--space-xs);
  padding: var(--space-xs) var(--space-sm);
  background: var(--color-surface-elevated);
  border-left: 2px solid var(--color-cta);
  border-radius: 0 var(--radius-sm) var(--radius-sm) 0;
}

.sequence-chain-title {
  font-size: 10px;
  font-weight: 600;
  color: var(--color-cta);
  display: block;
}

.sequence-chain-text {
  font-size: 11px;
  color: var(--color-text-muted);
  margin: 0;
  line-height: 1.4;
}

.task-meta {
  display: flex;
  align-items: center;
}

.task-priority {
  font-size: 11px;
  color: var(--color-text-dim);
  font-family: var(--font-heading);
  font-weight: 500;
}

.task-actions {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}

.action-play-btn {
  width: 28px;
  height: 28px;
  background: var(--color-cta);
  color: white;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.action-play-btn:hover:not(:disabled) {
  background: var(--color-cta-hover);
  transform: scale(1.05);
}

.action-play-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.action-stop-btn {
  width: 28px;
  height: 28px;
  background: var(--color-error);
  color: white;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  animation: pulse-stop 1.5s infinite;
}

@keyframes pulse-stop {
  0% { transform: scale(1); }
  50% { transform: scale(1.08); }
  100% { transform: scale(1); }
}

.icon-btn {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-sm);
  color: var(--color-text-dim);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.icon-btn:hover:not(:disabled) {
  background: var(--color-surface-elevated);
  color: var(--color-text);
}

.icon-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.icon-btn.danger:hover:not(:disabled) {
  background: rgba(239, 68, 68, 0.15);
  color: var(--color-error);
}

/* 编排弹窗 */
.editor-backdrop {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.editor-modal {
  width: 580px;
  max-width: 95%;
  max-height: 85vh;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 50px rgba(0, 0, 0, 0.3);
  animation: modalScale 0.25s cubic-bezier(0.34, 1.56, 0.64, 1);
}

@keyframes modalScale {
  from { transform: scale(0.95); opacity: 0; }
  to { transform: scale(1); opacity: 1; }
}

.modal-header {
  padding: var(--space-md) var(--space-lg);
  border-bottom: 1px solid var(--color-border);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.modal-header h3 {
  font-size: 15px;
  font-weight: 600;
  margin: 0;
}

.close-btn {
  color: var(--color-text-dim);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}

.close-btn:hover {
  color: var(--color-text);
}

.modal-body {
  padding: var(--space-lg);
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
  flex: 1;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
}

.form-group label {
  font-size: 12px;
  color: var(--color-text-dim);
  font-weight: 500;
}

.input {
  padding: var(--space-sm) var(--space-md);
  background: var(--color-surface-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  color: var(--color-text);
  font-size: 13px;
}

.input:focus {
  border-color: var(--color-cta);
}

.input.small {
  width: 100px;
}

/* 步骤容器 */
.steps-container {
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
  margin-top: var(--space-sm);
}

.steps-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.steps-header h4 {
  font-size: 12px;
  font-weight: 600;
  margin: 0;
  color: var(--color-text-dim);
}

.btn-add-step {
  display: flex;
  align-items: center;
  gap: 4px;
  background: rgba(51, 112, 255, 0.1);
  color: var(--color-cta);
  padding: var(--space-xs) var(--space-sm);
  border-radius: var(--radius-sm);
  font-size: 11px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-add-step:hover {
  background: var(--color-cta);
  color: white;
}

.steps-list {
  border: 1px dashed var(--color-border);
  border-radius: var(--radius-md);
  padding: var(--space-sm);
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
  background: var(--color-surface-elevated);
  max-height: 260px;
  overflow-y: auto;
}

.steps-empty {
  text-align: center;
  color: var(--color-text-dim);
  font-size: 12px;
  padding: var(--space-xl);
}

.step-editor-item {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: 6px var(--space-sm);
}

.step-index {
  font-size: 11px;
  font-weight: 600;
  color: var(--color-cta);
  width: 24px;
}

.step-select {
  flex: 1;
}

.select-input {
  width: 100%;
  padding: 6px var(--space-sm);
  background: var(--color-surface-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  color: var(--color-text);
  font-size: 12px;
  cursor: pointer;
}

.step-loop {
  display: flex;
  align-items: center;
  gap: 4px;
}

.loop-input {
  width: 60px;
  padding: 5px;
  background: var(--color-surface-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  color: var(--color-text);
  font-size: 12px;
  text-align: center;
}

.loop-label {
  font-size: 11px;
  color: var(--color-text-dim);
}

.step-actions {
  display: flex;
  gap: 2px;
}

.sort-btn,
.remove-btn {
  width: 22px;
  height: 22px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  color: var(--color-text-dim);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.sort-btn:hover:not(:disabled) {
  background: var(--color-surface-elevated);
  color: var(--color-text);
}

.sort-btn:disabled {
  opacity: 0.2;
  cursor: not-allowed;
}

.remove-btn:hover {
  background: rgba(239, 68, 68, 0.12);
  color: var(--color-error);
}

.modal-footer {
  padding: var(--space-md) var(--space-lg);
  border-top: 1px solid var(--color-border);
  display: flex;
  justify-content: flex-end;
  gap: var(--space-sm);
}

.btn-cancel {
  padding: var(--space-sm) var(--space-md);
  border-radius: var(--radius-md);
  font-size: 12px;
  font-weight: 500;
  background: transparent;
  color: var(--color-text-dim);
  border: 1px solid var(--color-border);
  cursor: pointer;
}

.btn-cancel:hover {
  border-color: var(--color-text);
  color: var(--color-text);
}

.btn-save {
  padding: var(--space-sm) var(--space-md);
  border-radius: var(--radius-md);
  font-size: 12px;
  font-weight: 500;
  background: var(--color-cta);
  color: white;
  cursor: pointer;
}

.btn-save:hover {
  background: var(--color-cta-hover);
}

/* 过渡动画 */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.scheduler-config-box {
  background: var(--color-surface-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: var(--space-md);
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
  margin-bottom: var(--space-sm);
}

.scheduler-config-title {
  font-size: 13px;
  font-weight: 600;
  margin: 0 0 var(--space-xs) 0;
  color: var(--color-cta);
}

.row-group {
  display: flex;
  gap: var(--space-md);
}

.row-group .field-item {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
}

.input-hint {
  font-size: 10px;
  color: var(--color-text-dim);
  margin-top: 2px;
}

.input-hint code {
  background: rgba(255, 255, 255, 0.05);
  padding: 1px 4px;
  border-radius: 3px;
  color: var(--color-cta);
}

.fade-in {
  animation: fadeIn 0.25s ease-out;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(3px); }
  to { opacity: 1; transform: translateY(0); }
}

.manual-info-box {
  display: flex;
  align-items: flex-start;
  gap: var(--space-sm);
  background: linear-gradient(135deg, rgba(99, 102, 241, 0.08) 0%, rgba(67, 56, 202, 0.05) 100%);
  border: 1px solid rgba(99, 102, 241, 0.3);
  border-radius: var(--radius-md);
  padding: var(--space-md);
}

.manual-info-icon {
  font-size: 20px;
  flex-shrink: 0;
  margin-top: 2px;
}

.manual-info-text {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.manual-info-text strong {
  font-size: 13px;
  color: var(--color-text);
}

.manual-info-text p {
  font-size: 12px;
  color: var(--color-text-dim);
  margin: 0;
  line-height: 1.5;
}

.notify-empty-hint {
  font-size: 11px;
  color: var(--color-text-dim);
  padding: var(--space-xs) 0;
}

.notify-checkbox-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: var(--space-sm);
  max-height: 120px;
  overflow-y: auto;
  margin-top: var(--space-xs);
}

.notify-checkbox-item {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  font-size: 12px;
  color: var(--color-text);
  cursor: pointer;
  padding: 2px 0;
}

.notify-checkbox-item input {
  cursor: pointer;
}

.notify-ch-name {
  font-weight: 500;
}

.notify-ch-type {
  color: var(--color-text-dim);
  font-size: 11px;
}
</style>

