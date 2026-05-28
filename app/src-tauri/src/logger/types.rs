use serde::{Deserialize, Serialize};
use chrono::DateTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: String,
    pub level: LogLevel,
    pub message: String,
    pub module: String,
    pub timestamp: DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogQuery {
    pub level: Option<LogLevel>,
    pub start: Option<DateTime<chrono::Utc>>,
    pub end: Option<DateTime<chrono::Utc>>,
    pub keyword: Option<String>,
    pub limit: Option<usize>,
}
