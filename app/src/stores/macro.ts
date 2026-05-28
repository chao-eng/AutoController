import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { MacroMeta, Macro } from '../types/macro'
import { invoke } from '@tauri-apps/api/core'

export const useMacroStore = defineStore('macro', () => {
  const macros = ref<MacroMeta[]>([])
  const isRecording = ref(false)
  const currentRecordingId = ref<string | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchMacros() {
    loading.value = true
    try {
      macros.value = await invoke<MacroMeta[]>('macro_list')
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function startRecord(deviceId: string, name: string) {
    try {
      const id = await invoke<string>('macro_start_record', { deviceId, name })
      isRecording.value = true
      currentRecordingId.value = id
      return id
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  async function stopRecord() {
    try {
      const macro = await invoke<Macro>('macro_stop_record')
      isRecording.value = false
      currentRecordingId.value = null
      await fetchMacros()
      return macro
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  async function play(macroId: string, speed = 1.0, loopCount = 1) {
    try {
      return await invoke<string>('macro_play', { macroId, speed, loopCount })
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  async function pause(playbackId: string) {
    try {
      await invoke('macro_pause', { playbackId })
    } catch (e) {
      error.value = String(e)
    }
  }

  async function resume(playbackId: string) {
    try {
      await invoke('macro_resume', { playbackId })
    } catch (e) {
      error.value = String(e)
    }
  }

  async function stop(playbackId: string) {
    try {
      await invoke('macro_stop', { playbackId })
    } catch (e) {
      error.value = String(e)
    }
  }

  async function deleteMacro(macroId: string) {
    try {
      await invoke('macro_delete', { macroId })
      macros.value = macros.value.filter((m) => m.id !== macroId)
    } catch (e) {
      error.value = String(e)
    }
  }

  return {
    macros,
    isRecording,
    currentRecordingId,
    loading,
    error,
    fetchMacros,
    startRecord,
    stopRecord,
    play,
    pause,
    resume,
    stop,
    deleteMacro,
  }
})
