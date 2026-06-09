use std::collections::HashSet;
use std::sync::Arc;

use parking_lot::{Condvar, Mutex};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use super::cancellation::CancellationToken;
use super::types::ScriptLineChangeEvent;

#[derive(Debug, Clone, Serialize)]
pub struct ScriptDebugEvent {
    pub execution_id: String,
    pub script_id: String,
    pub status: String,
    pub line: usize,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScriptDebugWatchEvent {
    pub execution_id: String,
    pub script_id: String,
    pub name: String,
    pub value: String,
    pub line: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DebugMode {
    Running,
    Step,
}

#[derive(Debug)]
struct DebugState {
    breakpoints: HashSet<usize>,
    mode: DebugMode,
    paused: bool,
    stop_requested: bool,
    current_line: usize,
}

#[derive(Clone, Debug)]
pub(super) struct DebugControl {
    state: Arc<(Mutex<DebugState>, Condvar)>,
}

impl DebugControl {
    pub fn new(breakpoints: Vec<usize>) -> Self {
        let mut breakpoint_set = HashSet::new();
        for line in breakpoints {
            if line > 0 {
                breakpoint_set.insert(line);
            }
        }

        Self {
            state: Arc::new((
                Mutex::new(DebugState {
                    breakpoints: breakpoint_set,
                    mode: DebugMode::Running,
                    paused: false,
                    stop_requested: false,
                    current_line: 0,
                }),
                Condvar::new(),
            )),
        }
    }

    pub fn resume(&self) {
        let (lock, wake) = &*self.state;
        let mut state = lock.lock();
        state.mode = DebugMode::Running;
        state.paused = false;
        wake.notify_all();
    }

    pub fn step(&self) {
        let (lock, wake) = &*self.state;
        let mut state = lock.lock();
        state.mode = DebugMode::Step;
        state.paused = false;
        wake.notify_all();
    }

    pub fn stop(&self) {
        let (lock, wake) = &*self.state;
        let mut state = lock.lock();
        state.stop_requested = true;
        state.paused = false;
        wake.notify_all();
    }

    pub fn current_line(&self) -> usize {
        let (lock, _) = &*self.state;
        lock.lock().current_line
    }

    pub fn hit(
        &self,
        line: usize,
        token: &CancellationToken,
        app_handle: &Arc<Mutex<Option<AppHandle>>>,
        execution_id: &str,
        script_id: &str,
    ) {
        if line == 0 || token.is_cancelled() {
            return;
        }

        emit_line(app_handle, execution_id, script_id, line);

        let should_pause = {
            let (lock, _) = &*self.state;
            let mut state = lock.lock();
            state.current_line = line;

            if state.stop_requested {
                return;
            }

            state.mode == DebugMode::Step || state.breakpoints.contains(&line)
        };

        if !should_pause {
            return;
        }

        emit_debug(
            app_handle,
            execution_id,
            script_id,
            "paused",
            line,
            Some(format!("暂停在第 {} 行", line)),
        );

        let (lock, wake) = &*self.state;
        let mut state = lock.lock();
        state.paused = true;
        while state.paused && !state.stop_requested && !token.is_cancelled() {
            wake.wait(&mut state);
        }
    }
}

pub(super) fn emit_debug(
    app_handle: &Arc<Mutex<Option<AppHandle>>>,
    execution_id: &str,
    script_id: &str,
    status: &str,
    line: usize,
    message: Option<String>,
) {
    let handle_guard = app_handle.lock();
    if let Some(ref handle) = *handle_guard {
        let _ = handle.emit(
            "script-debug",
            ScriptDebugEvent {
                execution_id: execution_id.to_string(),
                script_id: script_id.to_string(),
                status: status.to_string(),
                line,
                message,
            },
        );
    }
}

pub(super) fn emit_debug_watch(
    app_handle: &Arc<Mutex<Option<AppHandle>>>,
    execution_id: &str,
    script_id: &str,
    name: String,
    value: String,
    line: usize,
) {
    let handle_guard = app_handle.lock();
    if let Some(ref handle) = *handle_guard {
        let _ = handle.emit(
            "script-debug-watch",
            ScriptDebugWatchEvent {
                execution_id: execution_id.to_string(),
                script_id: script_id.to_string(),
                name,
                value,
                line,
            },
        );
    }
}

fn emit_line(
    app_handle: &Arc<Mutex<Option<AppHandle>>>,
    execution_id: &str,
    script_id: &str,
    line: usize,
) {
    let handle_guard = app_handle.lock();
    if let Some(ref handle) = *handle_guard {
        let _ = handle.emit(
            "script-line-change",
            ScriptLineChangeEvent {
                execution_id: execution_id.to_string(),
                script_id: script_id.to_string(),
                line,
            },
        );
    }
}

pub(super) fn instrument_script(code: &str) -> String {
    let mut out = String::with_capacity(code.len() + code.lines().count() * 20);

    for (idx, line) in code.lines().enumerate() {
        let line_number = idx + 1;
        let trimmed = line.trim_start();
        if should_instrument(trimmed) {
            let indent_len = line.len() - trimmed.len();
            out.push_str(&line[..indent_len]);
            out.push_str(&format!("__debug_hit({}); ", line_number));
            out.push_str(trimmed);
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }

    out
}

fn should_instrument(trimmed: &str) -> bool {
    !trimmed.is_empty()
        && !trimmed.starts_with("//")
        && !trimmed.starts_with("/*")
        && !trimmed.starts_with('*')
        && !trimmed.starts_with("fn ")
        && !trimmed.starts_with('}')
        && !trimmed.starts_with("else")
        && !trimmed.starts_with("catch")
}

#[cfg(test)]
mod tests {
    use super::{instrument_script, DebugControl, DebugMode};

    #[test]
    fn debug_control_starts_by_running_to_breakpoint() {
        let control = DebugControl::new(vec![5]);
        let (lock, _) = &*control.state;
        let state = lock.lock();

        assert_eq!(state.mode, DebugMode::Running);
        assert!(state.breakpoints.contains(&5));
    }

    #[test]
    fn instruments_executable_lines_without_breaking_else_chains() {
        let code = r#"let x = 1;
if x > 0 {
  log("yes");
} else {
  log("no");
}"#;

        let instrumented = instrument_script(code);

        assert!(instrumented.contains("__debug_hit(1); let x = 1;"));
        assert!(instrumented.contains("__debug_hit(2); if x > 0 {"));
        assert!(instrumented.contains("__debug_hit(3); log(\"yes\");"));
        assert!(!instrumented.contains("__debug_hit(4);"));
        assert!(instrumented.contains("__debug_hit(5); log(\"no\");"));
    }

    #[test]
    fn leaves_function_declarations_parseable() {
        let code = r#"fn main() {
  let x = 1;
}

main();"#;

        let instrumented = instrument_script(code);
        let mut engine = rhai::Engine::new();
        engine.register_fn("__debug_hit", |_line: i64| {});

        assert!(!instrumented.contains("__debug_hit(1); fn main"));
        assert!(engine.eval::<()>(&instrumented).is_ok());
    }
}
