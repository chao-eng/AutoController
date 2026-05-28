// 已移除 ControllerType 类型定义，仅模拟 Xbox 360 手柄

export enum Button {
  A = 'A',
  B = 'B',
  X = 'X',
  Y = 'Y',
  LB = 'LB',
  RB = 'RB',
  LT = 'LT',
  RT = 'RT',
  Back = 'Back',
  Start = 'Start',
  Guide = 'Guide',
  LeftThumb = 'LeftThumb',
  RightThumb = 'RightThumb',
  DPadUp = 'DPadUp',
  DPadDown = 'DPadDown',
  DPadLeft = 'DPadLeft',
  DPadRight = 'DPadRight',
}

export enum ThumbAxis {
  LeftX = 'LeftX',
  LeftY = 'LeftY',
  RightX = 'RightX',
  RightY = 'RightY',
}

export enum TriggerSide {
  Left = 'Left',
  Right = 'Right',
}

export interface ControllerState {
  buttons: number
  left_thumb_x: number
  left_thumb_y: number
  right_thumb_x: number
  right_thumb_y: number
  left_trigger: number
  right_trigger: number
}

export interface DeviceInfo {
  id: string
  index: number
  connected: boolean
  state: ControllerState
  vigem_connected: boolean
}

export interface ViGEmStatus {
  available: boolean
  driver_installed: boolean
  dll_found: boolean
  connected: boolean
  error_code: number | null
  message: string
}
