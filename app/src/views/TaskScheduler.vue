<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useSchedulerStore } from '../stores/scheduler'
import { useScriptStore } from '../stores/script'
import { useUIStore } from '../stores/ui'
import { useConfigStore } from '../stores/config'
import { Plus, ToggleLeft, ToggleRight, Trash2, Play, Square, Edit } from '@lucide/vue'
import type { ScheduledTask } from '../types/scheduler'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Checkbox } from '@/components/ui/checkbox'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'

const store = useSchedulerStore()
const scriptStore = useScriptStore()
const uiStore = useUIStore()
const configStore = useConfigStore()

const showEditor = ref(false)
const editingTaskId = ref<string | null>(null)
const taskName = ref('')
const taskLoopCount = ref(1)
const steps = ref<{ script_id: string; loop_count: number }[]>([])
const selectedNotificationChannels = ref<string[]>([])

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
  editingTaskId.value = null
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
    steps.value.push({
      script_id: scriptStore.scripts[0].id,
      loop_count: 1,
    })
  }
  showEditor.value = true
}

function openEditEditor(task: ScheduledTask) {
  editingTaskId.value = task.id
  taskName.value = task.name
  
  if (task.action && 'execute_sequence' in task.action) {
    taskLoopCount.value = task.action.execute_sequence.task_loop_count
    steps.value = task.action.execute_sequence.steps.map(s => ({
      script_id: s.script_id,
      loop_count: s.loop_count
    }))
  } else {
    taskLoopCount.value = 1
    steps.value = []
  }

  selectedNotificationChannels.value = task.notification_channels ? [...task.notification_channels] : []

  if (task.schedule === 'manual') {
    scheduleType.value = 'manual'
  } else if ('once' in task.schedule) {
    scheduleType.value = 'once'
    const date = new Date(task.schedule.once)
    const tzoffset = date.getTimezoneOffset() * 60000;
    const localISOTime = (new Date(date.getTime() - tzoffset)).toISOString().slice(0, 16);
    onceDateTime.value = localISOTime
  } else if ('daily' in task.schedule) {
    scheduleType.value = 'daily'
    dailyTime.value = task.schedule.daily.time
  } else if ('interval' in task.schedule) {
    scheduleType.value = 'interval'
    const ms = task.schedule.interval.duration_ms
    if (ms % 3600000 === 0) {
      intervalDuration.value = ms / 3600000
      intervalUnit.value = 'hours'
    } else if (ms % 60000 === 0) {
      intervalDuration.value = ms / 60000
      intervalUnit.value = 'minutes'
    } else {
      intervalDuration.value = ms / 1000
      intervalUnit.value = 'seconds'
    }
  } else if ('cron' in task.schedule) {
    scheduleType.value = 'cron'
    cronExpression.value = task.schedule.cron.expression
  }

  taskPriority.value = task.priority
  showEditor.value = true
}

function addStep() {
  if (scriptStore.scripts.length > 0) {
    steps.value.push({
      script_id: scriptStore.scripts[0].id,
      loop_count: 1,
    })
  } else {
    uiStore.showAlert('提示', '请先在"脚本"页面创建一些脚本后再添加步骤！')
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
    schedule = 'manual';
  }

  if (editingTaskId.value) {
    const originalTask = store.tasks.find(t => t.id === editingTaskId.value)
    if (!originalTask) return

    const updatedTask: ScheduledTask = {
      id: originalTask.id,
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
      enabled: originalTask.enabled,
      last_run: originalTask.last_run,
      next_run: originalTask.next_run,
      notification_channels: [...selectedNotificationChannels.value],
    }

    try {
      await store.updateTask(updatedTask)
      showEditor.value = false
      editingTaskId.value = null
      taskName.value = ''
      taskLoopCount.value = 1
      steps.value = []
      selectedNotificationChannels.value = []
      uiStore.showToast('修改任务序列成功', 'success')
    } catch (e) {
      uiStore.showAlert('修改失败', `修改任务序列失败: ${e}`)
    }
    return
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
  <div class="h-full overflow-y-auto p-6">
    <div class="flex items-center justify-between mb-6">
      <h2 class="text-lg font-semibold">任务调度与序列控制</h2>
      <Button variant="default" size="sm" @click="openEditor" :disabled="store.executingSequence">
        <Plus :size="14" />
        <span>新建任务序列</span>
      </Button>
    </div>

    <div v-if="store.executingSequence && store.sequenceProgress" class="bg-gradient-to-br from-green-500/10 via-transparent to-green-900/5 border border-green-500/25 rounded-xl p-4 mb-6 shadow-lg animate-in slide-in-from-top-2 duration-300">
      <div class="flex items-center justify-between mb-3">
        <div class="flex items-center gap-2">
          <span class="size-2 rounded-full bg-green-500 animate-pulse"></span>
          <h4 class="text-sm font-semibold text-green-500 m-0">串联任务序列正在运行...</h4>
        </div>
        <Button variant="destructive" size="sm" class="h-7 text-[11px]" @click="stopSequence(store.executingTaskId!)">
          <Square :size="12" fill="currentColor" />
          <span>强制终止序列</span>
        </Button>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-1">
        <div class="bg-card border border-border rounded-lg p-3">
          <div class="flex items-center justify-between text-xs text-muted-foreground mb-1">
            <span>序列总体循环</span>
            <span class="font-semibold text-green-500">{{ store.sequenceProgress.current_task_loop }} / {{ store.sequenceProgress.total_task_loops }} 轮</span>
          </div>
          <div class="h-1.5 bg-muted rounded-full overflow-hidden">
            <div class="h-full rounded-full bg-gradient-to-r from-emerald-400 to-emerald-600 transition-all duration-300" :style="{ width: `${(store.sequenceProgress.current_task_loop / store.sequenceProgress.total_task_loops) * 100}%` }"></div>
          </div>
        </div>
        <div class="bg-card border border-border rounded-lg p-3">
          <div class="flex items-center justify-between text-xs text-muted-foreground mb-1">
            <span>当前脚本: <strong class="text-foreground font-semibold">{{ store.sequenceProgress.current_script_name }}</strong></span>
            <span class="font-semibold text-blue-500">{{ store.sequenceProgress.current_step_loop }} / {{ store.sequenceProgress.total_step_loops }} 次</span>
          </div>
          <div class="h-1.5 bg-muted rounded-full overflow-hidden">
            <div class="h-full rounded-full bg-gradient-to-r from-blue-400 to-blue-600 transition-all duration-300" :style="{ width: `${(store.sequenceProgress.current_step_loop / store.sequenceProgress.total_step_loops) * 100}%` }"></div>
          </div>
        </div>
      </div>

      <div class="text-[11px] text-muted-foreground text-right">
        正在执行第 {{ store.sequenceProgress.current_step_index + 1 }} / {{ store.sequenceProgress.total_steps }} 个脚本步骤
      </div>
    </div>

    <div class="flex flex-col gap-2">
      <div v-if="store.tasks.length === 0" class="text-center text-muted-foreground py-12 text-sm">
        暂无串联任务，点击右上角"新建任务序列"开始设计编排吧
      </div>

      <div v-for="task in store.tasks" :key="task.id"
        class="flex items-center gap-4 bg-card border border-border rounded-xl p-4 transition-all hover:border-primary/50 hover:shadow-md"
        :class="{ 'opacity-50': !task.enabled, 'border-primary bg-gradient-to-br from-primary/[0.03] to-transparent': store.executingTaskId === task.id }"
      >
        <div class="flex-1 flex flex-col gap-1 min-w-0">
          <div class="flex items-center gap-2">
            <span class="text-sm font-semibold">{{ task.name }}</span>
            <Badge v-if="hasSequence(task.action)" variant="secondary" class="text-[10px] h-5 text-primary bg-primary/15">序列</Badge>
          </div>
          <span class="text-[11px] text-muted-foreground">{{ getScheduleLabel(task.schedule) }}</span>
          <span class="text-[11px] text-muted-foreground/70">{{ getActionLabel(task.action) }}</span>

          <div v-if="hasSequence(task.action)" class="mt-1 bg-muted/50 border-l-2 border-primary rounded-r-md px-2 py-1">
            <span class="text-[10px] font-semibold text-primary block">串联链条:</span>
            <p class="text-[11px] text-muted-foreground m-0 leading-relaxed">{{ getSequenceSummary(task.action) }}</p>
          </div>
        </div>

        <div class="flex items-center shrink-0">
          <span class="text-[11px] text-muted-foreground font-medium">轮数: {{ getSequenceLoopCount(task.action) }}</span>
        </div>

        <div class="flex items-center gap-2">
          <Button
            v-if="store.executingTaskId !== task.id"
            variant="default"
            size="icon"
            class="size-7 rounded-full"
            @click="startSequence(task.id)"
            :disabled="store.executingSequence || !task.enabled"
            title="运行串联序列"
          >
            <Play :size="14" fill="currentColor" />
          </Button>
          <Button
            v-else
            variant="destructive"
            size="icon"
            class="size-7 rounded-full animate-pulse"
            @click="stopSequence(task.id)"
            title="终止运行"
          >
            <Square :size="14" fill="currentColor" />
          </Button>

          <Button
            variant="ghost"
            size="icon"
            class="size-7"
            @click="store.toggleTask(task.id, !task.enabled)"
            :disabled="store.executingTaskId === task.id"
            :title="task.enabled ? '禁用' : '启用'"
          >
            <ToggleRight v-if="task.enabled" :size="18" class="text-primary" />
            <ToggleLeft v-else :size="18" />
          </Button>

          <Button
            variant="ghost"
            size="icon"
            class="size-7"
            @click="openEditEditor(task)"
            :disabled="store.executingTaskId === task.id"
            title="编辑任务"
          >
            <Edit :size="14" />
          </Button>

          <Button
            variant="ghost"
            size="icon"
            class="size-7 text-destructive hover:text-destructive hover:bg-destructive/10"
            @click="store.removeTask(task.id)"
            :disabled="store.executingTaskId === task.id"
            title="删除"
          >
            <Trash2 :size="14" />
          </Button>
        </div>
      </div>
    </div>

    <Dialog :open="showEditor" @update:open="showEditor = $event">
      <DialogContent class="sm:max-w-[580px] max-h-[85vh] overflow-hidden flex flex-col">
        <DialogHeader>
          <DialogTitle>{{ editingTaskId ? '修改任务序列与调度' : '编排串联多脚本序列' }}</DialogTitle>
          <DialogDescription class="sr-only">{{ editingTaskId ? '修改现有任务序列的名称、调度和执行步骤' : '创建新的任务序列，编排多个脚本的执行顺序与调度' }}</DialogDescription>
        </DialogHeader>

        <div class="flex-1 overflow-y-auto -mx-6 px-6 space-y-4">
          <div class="space-y-2">
            <Label class="text-xs font-medium text-muted-foreground">序列名称</Label>
            <Input v-model="taskName" placeholder="输入任务序列名称 (例如: 刷圈结算一体化)" />
          </div>

          <div class="space-y-2">
            <Label class="text-xs font-medium text-muted-foreground">整个任务的执行轮数 (Loop Count)</Label>
            <Input v-model.number="taskLoopCount" type="number" min="1" max="999" class="w-[100px]" />
          </div>

          <div class="bg-muted/30 border border-border rounded-lg p-4 space-y-3">
            <h4 class="text-sm font-semibold text-primary m-0">⏰ 任务调度配置</h4>

            <div class="space-y-2">
              <Label class="text-xs font-medium text-muted-foreground">调度类型 (Schedule Type)</Label>
              <Select v-model="scheduleType">
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="manual">🖱️ 手动执行 (Manual) — 不自动触发</SelectItem>
                  <SelectItem value="once">📅 单次定时执行 (Once)</SelectItem>
                  <SelectItem value="daily">🕒 每日固定时间 (Daily)</SelectItem>
                  <SelectItem value="interval">🔁 周期循环间隔 (Interval)</SelectItem>
                  <SelectItem value="cron">⚡ 标准 Cron 表达式 (Cron)</SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div v-if="scheduleType === 'manual'" class="flex items-start gap-3 bg-indigo-500/10 border border-indigo-500/30 rounded-lg p-4 animate-in fade-in duration-200">
              <span class="text-xl shrink-0 mt-0.5">🖱️</span>
              <div>
                <strong class="text-sm text-foreground">手动执行模式</strong>
                <p class="text-xs text-muted-foreground m-0 mt-1 leading-relaxed">系统调度器不会自动触发此任务。仅在你点击任务卡片上的 <strong>▶ 运行</strong> 按钮时执行一次。</p>
              </div>
            </div>

            <div v-if="scheduleType === 'once'" class="space-y-2 animate-in fade-in duration-200">
              <Label class="text-xs font-medium text-muted-foreground">执行时间</Label>
              <Input v-model="onceDateTime" type="datetime-local" />
            </div>

            <div v-if="scheduleType === 'daily'" class="space-y-2 animate-in fade-in duration-200">
              <Label class="text-xs font-medium text-muted-foreground">每日固定时间 (时:分:秒)</Label>
              <Input v-model="dailyTime" type="time" step="1" />
            </div>

            <div v-if="scheduleType === 'interval'" class="flex gap-4 animate-in fade-in duration-200">
              <div class="flex-1 space-y-2">
                <Label class="text-xs font-medium text-muted-foreground">执行间隔</Label>
                <Input v-model.number="intervalDuration" type="number" min="1" />
              </div>
              <div class="flex-1 space-y-2">
                <Label class="text-xs font-medium text-muted-foreground">时间单位</Label>
                <Select v-model="intervalUnit">
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="seconds">秒 (Seconds)</SelectItem>
                    <SelectItem value="minutes">分钟 (Minutes)</SelectItem>
                    <SelectItem value="hours">小时 (Hours)</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>

            <div v-if="scheduleType === 'cron'" class="space-y-2 animate-in fade-in duration-200">
              <Label class="text-xs font-medium text-muted-foreground">标准 Cron 表达式 (5字段: 分 时 日 月 周)</Label>
              <Input v-model="cronExpression" placeholder="*/5 * * * * (每 5 分钟)" />
              <span class="text-[10px] text-muted-foreground">例如: <code class="bg-muted/50 text-primary px-1 rounded font-mono">0 12 * * *</code> (每日中午 12 点), <code class="bg-muted/50 text-primary px-1 rounded font-mono">*/30 * * * *</code> (每半小时)</span>
            </div>

            <div v-if="scheduleType !== 'manual'" class="space-y-2">
              <Label class="text-xs font-medium text-muted-foreground">调度抢占优先级 (1-100，数字越大优先级越高)</Label>
              <Input v-model.number="taskPriority" type="number" min="1" max="100" class="w-[100px]" />
            </div>
          </div>

          <div class="space-y-2">
            <div class="flex items-center justify-between">
              <h4 class="text-xs font-semibold text-muted-foreground m-0">串联执行步骤顺序 (顺序从上到下)</h4>
              <Button variant="outline" size="sm" class="h-7 text-[11px]" @click="addStep">
                <Plus :size="12" /> 添加串联步骤
              </Button>
            </div>

            <div class="border border-dashed border-border rounded-lg p-2 bg-muted/30 space-y-1 max-h-[260px] overflow-y-auto">
              <div v-if="steps.length === 0" class="text-center text-muted-foreground text-xs py-8">
                点击"添加串联步骤"绑定脚本
              </div>
              <div v-for="(step, index) in steps" :key="index" class="flex items-center gap-2 bg-card border border-border rounded-lg p-2">
                <span class="text-[11px] font-semibold text-primary w-6 shrink-0">#{{ index + 1 }}</span>

                <div class="flex-1">
                  <Select v-model="step.script_id">
                    <SelectTrigger class="h-8 text-xs">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem v-for="s in scriptStore.scripts" :key="s.id" :value="s.id">
                        {{ s.name }}
                      </SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                <div class="flex items-center gap-1">
                  <Input v-model.number="step.loop_count" type="number" min="1" max="999" class="w-[60px] h-8 text-xs text-center" />
                  <span class="text-[11px] text-muted-foreground">次</span>
                </div>

                <div class="flex gap-0.5">
                  <Button variant="ghost" size="icon" class="size-6 text-[10px]" :disabled="index === 0" @click="moveStepUp(index)" title="上移">▲</Button>
                  <Button variant="ghost" size="icon" class="size-6 text-[10px]" :disabled="index === steps.length - 1" @click="moveStepDown(index)" title="下移">▼</Button>
                  <Button variant="ghost" size="icon" class="size-6 text-destructive hover:text-destructive hover:bg-destructive/10" @click="removeStep(index)" title="移除">
                    <Trash2 :size="12" />
                  </Button>
                </div>
              </div>
            </div>
          </div>

          <div class="bg-muted/30 border border-border rounded-lg p-4 space-y-3">
            <h4 class="text-sm font-semibold text-primary m-0">🔔 任务通知配置</h4>
            <div class="space-y-2">
              <Label class="text-xs font-medium text-muted-foreground">选择通知通道 (任务完成或中断时发送通知)</Label>
              <div v-if="!configStore.config.notification_channels || configStore.config.notification_channels.length === 0" class="text-[11px] text-muted-foreground">
                暂无配置好的通知通道。你可以先去"通知配置"页面添加。
              </div>
              <div v-else class="flex flex-col gap-1 bg-card border border-border rounded-lg p-2 max-h-[120px] overflow-y-auto">
                <label v-for="ch in configStore.config.notification_channels" :key="ch.id" class="flex items-center gap-2 text-xs text-foreground cursor-pointer py-0.5">
                  <Checkbox
                    :checked="selectedNotificationChannels.includes(ch.id)"
                    @update:checked="(checked: boolean) => {
                      if (checked) selectedNotificationChannels.push(ch.id)
                      else selectedNotificationChannels = selectedNotificationChannels.filter(id => id !== ch.id)
                    }"
                  />
                  <span class="font-medium">{{ ch.name }}</span>
                  <span class="text-muted-foreground text-[11px]">({{ getTypeName(ch.config.type) }})</span>
                </label>
              </div>
            </div>
          </div>
        </div>

        <DialogFooter class="border-t border-border pt-4 -mx-6 px-6 mt-4">
          <Button variant="outline" size="sm" @click="showEditor = false">取消</Button>
          <Button variant="default" size="sm" @click="saveTask">保存序列</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
</template>