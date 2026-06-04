use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    pub id: String,
    pub name: String,
    pub code: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptMeta {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ScriptOutput {
    pub execution_id: String,
    pub level: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScriptExecutionEvent {
    pub execution_id: String,
    pub script_id: String,
    pub status: String,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScriptLineChangeEvent {
    pub execution_id: String,
    pub script_id: String,
    pub line: usize,
}

