export interface MacroEvent {
  timestamp_ms: number
  device_id: string
  event_type: MacroEventType
}

export type MacroEventType =
  | { ButtonPress: string }
  | { ButtonRelease: string }
  | { ThumbMove: [string, number, number] }
  | { TriggerMove: [string, number] }

export interface Macro {
  id: string
  name: string
  created_at: string
  total_duration_ms: number
  events: MacroEvent[]
}

export interface MacroMeta {
  id: string
  name: string
  created_at: string
  total_duration_ms: number
  event_count: number
}
