import { invoke } from '@tauri-apps/api/core'
import type { DeviceInfo, ControllerState, ViGEmStatus } from '../types/controller'
import { Button, ThumbAxis, TriggerSide } from '../types/controller'

export function useController() {
  async function createDevice(): Promise<DeviceInfo> {
    return invoke<DeviceInfo>('controller_create')
  }

  async function removeDevice(deviceId: string): Promise<void> {
    return invoke('controller_remove', { deviceId })
  }

  async function setButton(deviceId: string, button: Button, pressed: boolean): Promise<void> {
    return invoke('controller_set_button', { deviceId, button, pressed })
  }

  async function setThumb(deviceId: string, axis: ThumbAxis, value: number): Promise<void> {
    return invoke('controller_set_thumb', { deviceId, axis, value })
  }

  async function setTrigger(deviceId: string, trigger: TriggerSide, value: number): Promise<void> {
    return invoke('controller_set_trigger', { deviceId, trigger, value })
  }

  async function getState(deviceId: string): Promise<ControllerState> {
    return invoke<ControllerState>('controller_get_state', { deviceId })
  }

  async function listDevices(): Promise<DeviceInfo[]> {
    return invoke<DeviceInfo[]>('controller_list')
  }

  async function getViGEmStatus(): Promise<ViGEmStatus> {
    return invoke<ViGEmStatus>('controller_vigem_status')
  }

  async function toggleConnection(deviceId: string): Promise<DeviceInfo> {
    return invoke<DeviceInfo>('controller_toggle_connection', { deviceId })
  }

  return {
    createDevice,
    removeDevice,
    setButton,
    setThumb,
    setTrigger,
    getState,
    listDevices,
    getViGEmStatus,
    toggleConnection,
  }
}
