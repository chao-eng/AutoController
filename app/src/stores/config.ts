import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import type { AppConfig } from '../types/config'
import { invoke } from '@tauri-apps/api/core'

export const useConfigStore = defineStore('config', () => {
  const config = ref<AppConfig>({
    devices: [],
    profiles: [],
    active_profile: null,
    auto_start: false,
    minimize_to_tray: true,
    log_level: 'info',
    ocr_region: null,
    ocr_regions: [],
  })
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchConfig() {
    loading.value = true
    try {
      config.value = await invoke<AppConfig>('config_get')
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function saveConfig(newConfig?: AppConfig) {
    try {
      await invoke('config_set', { newConfig: newConfig || config.value })
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  watch(config, () => {
    saveConfig()
  }, { deep: true })

  return {
    config,
    loading,
    error,
    fetchConfig,
    saveConfig,
  }
})
