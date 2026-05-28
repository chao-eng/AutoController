export interface Script {
  id: string
  name: string
  code: string
  created_at: string
  updated_at: string
}

export interface ScriptMeta {
  id: string
  name: string
  created_at: string
  updated_at: string
}

export interface ScriptOutput {
  execution_id: string
  level: string
  message: string
  timestamp: string
}
