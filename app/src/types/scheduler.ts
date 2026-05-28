export type ScheduleType =
  | { once: string }
  | { daily: { time: string } }
  | { interval: { duration_ms: number } }
  | { cron: { expression: string } }
  | 'manual'

export interface ScriptStep {
  script_id: string
  loop_count: number
}

export type TaskAction =
  | { play_macro: { macro_id: string; speed: number; loop_count: number } }
  | { execute_script: { script_id: string } }
  | { execute_sequence: { steps: ScriptStep[]; task_loop_count: number } }

export interface ScheduledTask {
  id: string
  name: string
  schedule: ScheduleType
  action: TaskAction
  priority: number
  enabled: boolean
  last_run: string | null
  next_run: string | null
}
