export interface DeviceConfig {
  id: string
  enabled: boolean
}

export interface GameProfile {
  id: string
  name: string
  game_process: string
  macros: string[]
  scripts: string[]
}

export interface OcrRegion {
  x: number
  y: number
  w: number
  h: number
}

export interface NotificationChannel {
  id: string
  name: string
  config: {
    type: 'feishu' | 'serverchan' | 'serverchan3' | 'telegram'
    webhook_url?: string
    secret?: string
    uid?: string
    send_key?: string
    bot_token?: string
    chat_id?: string
  }
}

export interface AppConfig {
  devices: DeviceConfig[]
  profiles: GameProfile[]
  active_profile: string | null
  auto_start: boolean
  minimize_to_tray: boolean
  log_level: string
  ocr_region?: OcrRegion | null
  ocr_regions?: OcrRegion[]
  notification_channels?: NotificationChannel[]
}

