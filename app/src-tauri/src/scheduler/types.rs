use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScheduleType {
    Once(DateTime<Utc>),
    Daily { time: String },
    Interval { duration_ms: u64 },
    Cron { expression: String },
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptStep {
    pub script_id: String,
    pub loop_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskAction {
    PlayMacro { macro_id: String, speed: f32, loop_count: u32 },
    ExecuteScript { script_id: String },
    ExecuteSequence {
        steps: Vec<ScriptStep>,
        task_loop_count: u32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub id: String,
    pub name: String,
    pub schedule: ScheduleType,
    pub action: TaskAction,
    pub priority: u8,
    pub enabled: bool,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: Option<DateTime<Utc>>,
    #[serde(default)]
    pub notification_channels: Vec<String>,
}

