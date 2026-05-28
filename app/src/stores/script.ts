import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { ScriptMeta, Script } from '../types/script'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

interface ScriptExecutionEvent {
  execution_id: string
  script_id: string
  status: 'started' | 'completed' | 'error'
  message: string | null
}

export const useScriptStore = defineStore('script', () => {
  const scripts = ref<ScriptMeta[]>([])
  const currentScript = ref<Script | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  // 全局脚本运行状态
  const executing = ref(false)
  const executionId = ref<string | null>(null)
  const executionStatus = ref<'idle' | 'running' | 'success' | 'error'>('idle')
  const executionMessage = ref('')
  const activeLine = ref<number>(0)
  let unlisten: UnlistenFn | null = null
  let lineUnlisten: UnlistenFn | null = null

  async function fetchScripts() {
    loading.value = true
    try {
      scripts.value = await invoke<ScriptMeta[]>('script_list')
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function createScript(name: string, code: string) {
    try {
      const script = await invoke<Script>('script_create', { name, code })
      await fetchScripts()
      return script
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  async function getScript(scriptId: string) {
    try {
      currentScript.value = await invoke<Script>('script_get', { scriptId })
      return currentScript.value
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  async function updateScript(scriptId: string, code: string) {
    try {
      const script = await invoke<Script>('script_update', { scriptId, code })
      currentScript.value = script
      return script
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  async function renameScript(scriptId: string, newName: string) {
    try {
      const script = await invoke<Script>('script_rename', { scriptId, newName })
      if (currentScript.value?.id === scriptId) {
        currentScript.value = script
      }
      await fetchScripts()
      return script
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }


  async function executeScript(scriptId: string) {
    try {
      executing.value = true
      executionStatus.value = 'running'
      executionMessage.value = '正在启动脚本...'
      
      const eid = await invoke<string>('script_execute', { scriptId })
      executionId.value = eid
      return eid
    } catch (e) {
      error.value = String(e)
      executing.value = false
      executionId.value = null
      executionStatus.value = 'error'
      executionMessage.value = `启动失败: ${e}`
      throw e
    }
  }

  async function stopExecution(eid: string) {
    try {
      await invoke('script_stop', { executionId: eid })
      executing.value = false
      executionId.value = null
      executionStatus.value = 'idle'
      executionMessage.value = ''
    } catch (e) {
      error.value = String(e)
    }
  }

  async function deleteScript(scriptId: string) {
    try {
      await invoke('script_delete', { scriptId })
      scripts.value = scripts.value.filter((s) => s.id !== scriptId)
      if (currentScript.value?.id === scriptId) {
        currentScript.value = null
      }
    } catch (e) {
      error.value = String(e)
    }
  }

  // 全局事件监听器接口
  async function startListening() {
    if (unlisten) return
    try {
      unlisten = await listen<ScriptExecutionEvent>('script-execution', (event) => {
        const { execution_id, status, message } = event.payload

        // 如果目前正在执行，且事件是该执行ID，则处理
        if (executionId.value && execution_id !== executionId.value) return

        if (status === 'started') {
          executionStatus.value = 'running'
          executionMessage.value = message || '脚本开始执行'
        } else if (status === 'completed') {
          executing.value = false
          executionId.value = null
          executionStatus.value = 'success'
          executionMessage.value = message || '脚本执行完成'
          activeLine.value = 0
        } else if (status === 'error') {
          executing.value = false
          executionId.value = null
          executionStatus.value = 'error'
          executionMessage.value = message || '脚本执行出错'
          activeLine.value = 0
        }
      })

      lineUnlisten = await listen<{ execution_id: string; script_id: string; line: number }>('script-line-change', (event) => {
        const { script_id, line } = event.payload
        if (currentScript.value?.id === script_id) {
          activeLine.value = line
        }
      })
    } catch (e) {
      console.warn('监听 script-execution/line-change 事件失败:', e)
    }
  }

  function stopListening() {
    unlisten?.()
    unlisten = null
    lineUnlisten?.()
    lineUnlisten = null
    activeLine.value = 0
  }

  return {
    scripts,
    currentScript,
    loading,
    error,
    executing,
    executionId,
    executionStatus,
    executionMessage,
    activeLine,
    fetchScripts,
    createScript,
    getScript,
    updateScript,
    renameScript,
    executeScript,
    stopExecution,
    deleteScript,
    startListening,
    stopListening,
  }
})
