import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { DeviceInfo, ViGEmStatus, ControllerState } from '../types/controller'
import { useController } from '../composables/useController'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

interface ControllerStateEvent {
  device_id: string
  state: ControllerState
}

export const useControllerStore = defineStore('controller', () => {
  const devices = ref<DeviceInfo[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)
  const vigemStatus = ref<ViGEmStatus | null>(null)

  const controller = useController()
  let unlisten: UnlistenFn | null = null
  let pollTimer: ReturnType<typeof setInterval> | null = null
  let initialized = false

  async function init() {
    if (initialized) {
      await fetchDevices()
      return
    }
    initialized = true

    await fetchDevices()
    await fetchViGEmStatus()

    try {
      unlisten = await listen<ControllerStateEvent>('controller-state-changed', (event) => {
        const { device_id, state } = event.payload
        const device = devices.value.find((d) => d.id === device_id)
        if (device) {
          device.state = { ...state }
        }
      })
    } catch (e) {
      console.warn('controller-state-changed 事件监听失败，将使用轮询模式', e)
    }

    pollTimer = setInterval(() => {
      fetchDevices()
    }, 500)
  }

  function cleanup() {
    if (!initialized) return
    initialized = false
    unlisten?.()
    unlisten = null
    if (pollTimer) {
      clearInterval(pollTimer)
      pollTimer = null
    }
  }

  async function fetchDevices() {
    try {
      const list = await controller.listDevices()
      if (devices.value.length === 0) {
        devices.value = list
        return
      }
      for (const updated of list) {
        const existing = devices.value.find((d) => d.id === updated.id)
        if (existing) {
          existing.state = { ...updated.state }
          existing.connected = updated.connected
          existing.vigem_connected = updated.vigem_connected
        } else {
          devices.value.push(updated)
        }
      }
      devices.value = devices.value.filter((d) =>
        list.some((u) => u.id === d.id)
      )
      devices.value.sort((a, b) => a.index - b.index)
    } catch (e) {
      error.value = String(e)
    }
  }

  async function fetchViGEmStatus() {
    try {
      vigemStatus.value = await controller.getViGEmStatus()
    } catch (e) {
      error.value = String(e)
    }
  }

  async function createDevice() {
    try {
      const device = await controller.createDevice()
      devices.value.push(device)
      devices.value.sort((a, b) => a.index - b.index)
      return device
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  async function removeDevice(deviceId: string) {
    try {
      await controller.removeDevice(deviceId)
      devices.value = devices.value.filter((d) => d.id !== deviceId)
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  async function setButton(deviceId: string, button: string, pressed: boolean) {
    try {
      await controller.setButton(deviceId, button as any, pressed)
    } catch (e) {
      error.value = String(e)
    }
  }

  async function setThumb(deviceId: string, axis: string, value: number) {
    try {
      await controller.setThumb(deviceId, axis as any, value)
    } catch (e) {
      error.value = String(e)
    }
  }

  async function setTrigger(deviceId: string, trigger: string, value: number) {
    try {
      await controller.setTrigger(deviceId, trigger as any, value)
    } catch (e) {
      error.value = String(e)
    }
  }

  async function toggleConnection(deviceId: string) {
    try {
      const updated = await controller.toggleConnection(deviceId)
      const existing = devices.value.find((d) => d.id === deviceId)
      if (existing) {
        existing.connected = updated.connected
        existing.vigem_connected = updated.vigem_connected
      }
    } catch (e) {
      error.value = String(e)
    }
  }

  return {
    devices,
    loading,
    error,
    vigemStatus,
    init,
    cleanup,
    fetchDevices,
    fetchViGEmStatus,
    createDevice,
    removeDevice,
    setButton,
    setThumb,
    setTrigger,
    toggleConnection,
  }
})
