use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::controller::types::Button;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MacroEventType {
    ButtonPress(Button),
    ButtonRelease(Button),
    ThumbMove(String, f32, f32),
    TriggerMove(String, f32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroEvent {
    pub timestamp_ms: u64,
    pub device_id: String,
    pub event_type: MacroEventType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Macro {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub total_duration_ms: u64,
    pub events: Vec<MacroEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroMeta {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub total_duration_ms: u64,
    pub event_count: usize,
}
