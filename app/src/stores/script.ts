import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { ScriptMeta, Script } from '../types/script'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { preferenceKeys, readPreference, writePreference } from '@/lib/preferences'

interface ScriptExecutionEvent {
  execution_id: string
  script_id: string
  status: 'started' | 'completed' | 'error'
  message: string | null
}

interface ScriptDebugEvent {
  execution_id: string
  script_id: string
  status: 'started' | 'paused' | 'running' | 'stepping' | 'completed' | 'stopped' | 'error'
  line: number
  message: string | null
}

interface ScriptValidationResult {
  valid: boolean
  line: number
  column: number
  message: string | null
}

interface ScriptDebugWatchEvent {
  execution_id: string
  script_id: string
  name: string
  value: string
  line: number
}

interface ScriptDebugWatch {
  name: string
  value: string
  line: number
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
  const debugExecutionId = ref<string | null>(null)
  const debugStatus = ref<'idle' | 'running' | 'paused' | 'stepping'>('idle')
  const debugLine = ref<number>(0)
  const debugMessage = ref('')
  const debugWatches = ref<ScriptDebugWatch[]>([])
  let unlisten: UnlistenFn | null = null
  let lineUnlisten: UnlistenFn | null = null
  let debugUnlisten: UnlistenFn | null = null
  let debugWatchUnlisten: UnlistenFn | null = null

  async function fetchScripts() {
    loading.value = true
    try {
      const fetched = await invoke<ScriptMeta[]>('script_list')
      const orderedIds = readPreference<string[]>(preferenceKeys.scriptOrder, [])
      fetched.sort((a, b) => {
        let idxA = orderedIds.indexOf(a.id)
        let idxB = orderedIds.indexOf(b.id)
        if (idxA === -1) idxA = 99999
        if (idxB === -1) idxB = 99999
        return idxA - idxB
      })
      scripts.value = fetched
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

  async function validateCode(code: string) {
    try {
      return await invoke<ScriptValidationResult>('script_validate_code', { code })
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  function persistScriptOrder(orderedScripts: ScriptMeta[]) {
    writePreference(preferenceKeys.scriptOrder, orderedScripts.map((script) => script.id))
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
      activeLine.value = 0
      debugWatches.value = []
      
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

  async function debugScript(scriptId: string, breakpoints: number[]) {
    try {
      executing.value = true
      executionStatus.value = 'running'
      executionMessage.value = '正在启动调试...'
      debugStatus.value = 'running'
      debugLine.value = 0
      debugMessage.value = '调试会话启动中'
      activeLine.value = 0
      debugWatches.value = []

      const eid = await invoke<string>('script_debug_execute', { scriptId, breakpoints })
      executionId.value = eid
      debugExecutionId.value = eid
      return eid
    } catch (e) {
      error.value = String(e)
      executing.value = false
      executionId.value = null
      debugExecutionId.value = null
      debugStatus.value = 'idle'
      executionStatus.value = 'error'
      executionMessage.value = `调试启动失败: ${e}`
      throw e
    }
  }

  async function resumeDebug() {
    if (!debugExecutionId.value) return
    try {
      await invoke('script_debug_resume', { executionId: debugExecutionId.value })
      debugStatus.value = 'running'
    } catch (e) {
      error.value = String(e)
    }
  }

  async function stepDebug() {
    if (!debugExecutionId.value) return
    try {
      await invoke('script_debug_step', { executionId: debugExecutionId.value })
      debugStatus.value = 'stepping'
    } catch (e) {
      error.value = String(e)
    }
  }

  async function stopDebug() {
    if (!debugExecutionId.value) return
    try {
      await invoke('script_debug_stop', { executionId: debugExecutionId.value })
      executing.value = false
      executionId.value = null
      debugExecutionId.value = null
      debugStatus.value = 'idle'
      debugLine.value = 0
      debugMessage.value = ''
      activeLine.value = 0
      debugWatches.value = []
      executionStatus.value = 'idle'
      executionMessage.value = ''
    } catch (e) {
      error.value = String(e)
    }
  }

  async function stopExecution(eid: string) {
    try {
      await invoke('script_stop', { executionId: eid })
      executing.value = false
      executionId.value = null
      debugExecutionId.value = null
      debugStatus.value = 'idle'
      debugLine.value = 0
      debugMessage.value = ''
      executionStatus.value = 'idle'
      executionMessage.value = ''
      activeLine.value = 0
      debugWatches.value = []
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
          debugExecutionId.value = null
          debugStatus.value = 'idle'
          debugLine.value = 0
          debugMessage.value = ''
          debugWatches.value = []
          executionStatus.value = 'success'
          executionMessage.value = message || '脚本执行完成'
          activeLine.value = 0
        } else if (status === 'error') {
          executing.value = false
          executionId.value = null
          debugExecutionId.value = null
          debugStatus.value = 'idle'
          debugLine.value = 0
          debugMessage.value = ''
          debugWatches.value = []
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

      debugUnlisten = await listen<ScriptDebugEvent>('script-debug', (event) => {
        const { execution_id, script_id, status, line, message } = event.payload
        if (currentScript.value?.id && currentScript.value.id !== script_id) return
        if (debugExecutionId.value && execution_id !== debugExecutionId.value) return

        debugExecutionId.value = execution_id
        debugLine.value = line
        debugMessage.value = message || ''

        if (status === 'started' || status === 'running') {
          debugStatus.value = 'running'
        } else if (status === 'paused') {
          debugStatus.value = 'paused'
          activeLine.value = line
        } else if (status === 'stepping') {
          debugStatus.value = 'stepping'
        } else if (status === 'completed' || status === 'stopped') {
          debugExecutionId.value = null
          debugStatus.value = 'idle'
          debugLine.value = 0
          debugMessage.value = ''
          debugWatches.value = []
        } else if (status === 'error') {
          debugExecutionId.value = null
          debugStatus.value = 'idle'
          debugLine.value = 0
          debugMessage.value = message || '调试执行出错'
        }
      })

      debugWatchUnlisten = await listen<ScriptDebugWatchEvent>('script-debug-watch', (event) => {
        const { execution_id, script_id, name, value, line } = event.payload
        if (currentScript.value?.id && currentScript.value.id !== script_id) return
        if (executionId.value && execution_id !== executionId.value) return

        const next = debugWatches.value.filter((item) => item.name !== name)
        next.unshift({ name, value, line })
        debugWatches.value = next.slice(0, 12)
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
    debugUnlisten?.()
    debugUnlisten = null
    debugWatchUnlisten?.()
    debugWatchUnlisten = null
    activeLine.value = 0
    debugExecutionId.value = null
    debugStatus.value = 'idle'
    debugLine.value = 0
    debugMessage.value = ''
    debugWatches.value = []
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
    debugExecutionId,
    debugStatus,
    debugLine,
    debugMessage,
    debugWatches,
    fetchScripts,
    createScript,
    getScript,
    updateScript,
    validateCode,
    persistScriptOrder,
    renameScript,
    executeScript,
    debugScript,
    resumeDebug,
    stepDebug,
    stopDebug,
    stopExecution,
    deleteScript,
    startListening,
    stopListening,
  }
})
