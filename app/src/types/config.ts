export interface DeviceConfig {
  id: string
  controller_type: 'xbox360' | 'dual_shock4'
  enabled: boolean
}

export interface GameProfile {
  id: string
  name: string
  game_process: string
  controller_type: 'xbox360' | 'dual_shock4'
  macros: string[]
  scripts: string[]
}

export interface AppConfig {
  devices: DeviceConfig[]
  profiles: GameProfile[]
  active_profile: string | null
  auto_start: boolean
  minimize_to_tray: boolean
  log_level: string
}
