use parking_lot::Mutex;
use std::sync::Arc;
use tauri::Emitter;
use tracing_subscriber::Layer;

use super::types::*;

#[derive(Debug, Clone, serde::Serialize)]
struct LogEvent {
    id: String,
    level: String,
    message: String,
    module: String,
    timestamp: String,
}

pub struct TauriEventLayer {
    app_handle: Arc<Mutex<Option<tauri::AppHandle>>>,
}

impl TauriEventLayer {
    pub fn new() -> (Self, Arc<Mutex<Option<tauri::AppHandle>>>) {
        let handle: Arc<Mutex<Option<tauri::AppHandle>>> = Arc::new(Mutex::new(None));
        let layer = Self {
            app_handle: handle.clone(),
        };
        (layer, handle)
    }
}

impl<S> Layer<S> for TauriEventLayer
where
    S: tracing::Subscriber,
    S: for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let metadata = event.metadata();

        let level = match *metadata.level() {
            tracing::Level::ERROR => LogLevel::Error,
            tracing::Level::WARN => LogLevel::Warn,
            tracing::Level::INFO => LogLevel::Info,
            tracing::Level::DEBUG => LogLevel::Debug,
            tracing::Level::TRACE => LogLevel::Trace,
        };

        let mut visitor = MessageVisitor::new();
        event.record(&mut visitor);

        let module = metadata.module_path().unwrap_or("unknown").to_string();

        let log_event = LogEvent {
            id: uuid::Uuid::new_v4().to_string(),
            level: format!("{:?}", level),
            message: visitor.format(),
            module,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        let handle = self.app_handle.lock();
        if let Some(ref handle) = *handle {
            let _ = handle.emit("log-entry", &log_event);
        }
    }
}

struct MessageVisitor {
    message: String,
    fields: Vec<(String, String)>,
}

impl MessageVisitor {
    fn new() -> Self {
        Self {
            message: String::new(),
            fields: Vec::new(),
        }
    }

    fn format(self) -> String {
        if self.fields.is_empty() {
            return self.message;
        }

        let mut result = self.message;
        for (key, value) in self.fields {
            if !result.is_empty() {
                result.push(' ');
            }
            result.push_str(&format!("{}={}", key, value));
        }
        result
    }
}

impl tracing::field::Visit for MessageVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            use std::fmt::Write;
            let _ = write!(self.message, "{:?}", value);
        } else {
            use std::fmt::Write;
            let mut s = String::new();
            let _ = write!(s, "{:?}", value);
            self.fields.push((field.name().to_string(), s));
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message.push_str(value);
        } else {
            self.fields.push((field.name().to_string(), value.to_string()));
        }
    }

    fn record_error(
        &mut self,
        field: &tracing::field::Field,
        value: &(dyn std::error::Error + 'static),
    ) {
        if field.name() == "message" {
            self.message.push_str(&value.to_string());
        } else {
            self.fields.push((field.name().to_string(), value.to_string()));
        }
    }
}
