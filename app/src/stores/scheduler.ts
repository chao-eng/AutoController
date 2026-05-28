import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { ScheduledTask } from '../types/scheduler'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

export interface SequenceProgress {
  task_id: string
  running: boolean
  current_task_loop: number
  total_task_loops: number
  current_step_index: number
  total_steps: number
  current_step_loop: number
  total_step_loops: number
  current_script_name: string
}

export const useSchedulerStore = defineStore('scheduler', () => {
  const tasks = ref<ScheduledTask[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  // 任务序列运行状态
  const executingTaskId = ref<string | null>(null)
  const executingSequence = ref(false)
  const sequenceProgress = ref<SequenceProgress | null>(null)
  let unlisten: UnlistenFn | null = null

  async function fetchTasks() {
    loading.value = true
    try {
      tasks.value = await invoke<ScheduledTask[]>('scheduler_list')
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function createTask(task: ScheduledTask) {
    try {
      await invoke('scheduler_create_task', { task })
      await fetchTasks()
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  async function removeTask(taskId: string) {
    try {
      await invoke('scheduler_remove_task', { taskId })
      tasks.value = tasks.value.filter((t) => t.id !== taskId)
    } catch (e) {
      error.value = String(e)
    }
  }

  async function toggleTask(taskId: string, enabled: boolean) {
    try {
      await invoke('scheduler_toggle_task', { taskId, enabled })
      const task = tasks.value.find((t) => t.id === taskId)
      if (task) {
        task.enabled = enabled
      }
    } catch (e) {
      error.value = String(e)
    }
  }

  async function executeSequence(taskId: string) {
    try {
      executingTaskId.value = taskId
      executingSequence.value = true
      await invoke('scheduler_execute_sequence', { taskId })
    } catch (e) {
      error.value = String(e)
      executingTaskId.value = null
      executingSequence.value = false
      throw e
    }
  }

  async function stopSequence(taskId: string) {
    try {
      await invoke('scheduler_stop_sequence', { taskId })
      executingTaskId.value = null
      executingSequence.value = false
      sequenceProgress.value = null
    } catch (e) {
      error.value = String(e)
    }
  }

  async function startListening() {
    if (unlisten) return
    try {
      unlisten = await listen<SequenceProgress>('sequence-execution-progress', (event) => {
        const progress = event.payload
        if (!progress.running) {
          executingTaskId.value = null
          executingSequence.value = false
          sequenceProgress.value = null
        } else {
          executingTaskId.value = progress.task_id
          executingSequence.value = true
          sequenceProgress.value = progress
        }
      })
    } catch (e) {
      console.warn('监听 sequence-execution-progress 事件失败:', e)
    }
  }

  function stopListening() {
    unlisten?.()
    unlisten = null
  }

  return {
    tasks,
    loading,
    error,
    executingTaskId,
    executingSequence,
    sequenceProgress,
    fetchTasks,
    createTask,
    removeTask,
    toggleTask,
    executeSequence,
    stopSequence,
    startListening,
    stopListening,
  }
})
