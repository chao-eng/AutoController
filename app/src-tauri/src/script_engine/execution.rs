use std::sync::Arc;
use std::time::Duration;
use parking_lot::Mutex;
use tauri::Emitter;

use rhai::Engine;

use super::runtime::ScriptRuntime;
use super::cancellation::CancellationToken;
use super::types::{ScriptExecutionEvent, ScriptLineChangeEvent};

pub(super) struct Execution {
    pub(super) running: bool,
    pub(super) success: bool,
    pub(super) error: Option<String>,
    pub(super) token: CancellationToken,
}

impl ScriptRuntime {
    pub fn execute_script(&self, script_id: &str) -> Result<String, String> {
        let (code, name) = {
            let scripts = self.scripts.lock();
            let script = scripts
                .get(script_id)
                .ok_or_else(|| format!("脚本不存在: {}", script_id))?;
            (script.code.clone(), script.name.clone())
        };

        let execution_id = uuid::Uuid::new_v4().to_string();
        let eid = execution_id.clone();
        let sid = script_id.to_string();

        let token = CancellationToken::new();

        {
            let mut executions = self.executions.lock();
            executions.insert(
                execution_id.clone(),
                Execution {
                    running: true,
                    success: false,
                    error: None,
                    token: token.clone(),
                },
            );
        }

        let controller = self.controller.clone();
        let executions = self.executions.clone();
        let app_handle = self.app_handle.clone();
        let token_thread = token.clone();

        {
            let app_handle_guard = app_handle.lock();
            if let Some(ref handle) = *app_handle_guard {
                let _ = handle.emit(
                    "script-execution",
                    ScriptExecutionEvent {
                        execution_id: execution_id.clone(),
                        script_id: script_id.to_string(),
                        status: "started".to_string(),
                        message: Some(format!("脚本 '{}' 开始执行", name)),
                    },
                );
            }
        }

        std::thread::spawn(move || {
            tracing::info!(execution_id = %eid, script_id = %sid, name = %name, "脚本开始执行");

            let mut engine = Engine::new();
            engine.set_allow_looping(true);
            engine.set_max_operations(10_000_000);
            engine.set_max_string_size(100_000);
            engine.set_max_array_size(10_000);

            engine.register_fn("to_int", move |s: &str| -> i64 {
                s.trim().parse::<i64>().unwrap_or(0)
            });

            let default_device = Arc::new(Mutex::new("0".to_string()));

            // Register controller bindings
            super::controller_bindings::register_controller_bindings(
                &mut engine,
                controller.clone(),
                default_device,
                app_handle.clone(),
                eid.clone(),
                sid.clone(),
            );

            // Register OCR bindings
            super::ocr_bindings::register_ocr_bindings(
                &mut engine,
                app_handle.clone(),
                eid.clone(),
                sid.clone(),
            );

            // Register Telemetry bindings
            super::telemetry_bindings::register_telemetry_bindings(
                &mut engine,
            );

            let token_progress = token_thread.clone();
            engine.on_progress(move |_ops| {
                if token_progress.is_cancelled() {
                    Some(rhai::Dynamic::from("脚本执行已停止".to_string()))
                } else {
                    None
                }
            });

            let token_sleep = token_thread.clone();
            let handle_sleep = app_handle.clone();
            let eid_sleep = eid.clone();
            let sid_sleep = sid.clone();
            engine.register_fn("sleep", move |context: rhai::NativeCallContext, ms: i64| {
                {
                    let handle_guard = handle_sleep.lock();
                    if let Some(ref handle) = *handle_guard {
                        if let Some(line) = context.call_position().line() {
                            let _ = handle.emit(
                                "script-line-change",
                                ScriptLineChangeEvent {
                                    execution_id: eid_sleep.clone(),
                                    script_id: sid_sleep.clone(),
                                    line,
                                },
                            );
                        }
                    }
                }
                let total = ms as u64;
                let step = 50;
                let mut elapsed = 0;
                while elapsed < total {
                    if token_sleep.is_cancelled() {
                        break;
                    }
                    let remaining = total - elapsed;
                    let to_sleep = if remaining < step { remaining } else { step };
                    std::thread::sleep(Duration::from_millis(to_sleep));
                    elapsed += to_sleep;
                }
            });

            let handle_log = app_handle.clone();
            let eid_log = eid.clone();
            let sid_log = sid.clone();
            engine.register_fn("log", move |context: rhai::NativeCallContext, msg: &str| {
                {
                    let handle_guard = handle_log.lock();
                    if let Some(ref handle) = *handle_guard {
                        if let Some(line) = context.call_position().line() {
                            let _ = handle.emit(
                                "script-line-change",
                                ScriptLineChangeEvent {
                                    execution_id: eid_log.clone(),
                                    script_id: sid_log.clone(),
                                    line,
                                },
                            );
                        }
                    }
                }
                tracing::info!(target: "script", "[脚本] {}", msg);
            });

            let wrapped_code = Self::wrap_script(&code);

            let (success, err_msg) = match engine.eval::<()>(&wrapped_code) {
                Ok(()) => {
                    tracing::info!(execution_id = %eid, script_id = %sid, "脚本执行完成");
                    {
                        let handle_guard = app_handle.lock();
                        if let Some(ref handle) = *handle_guard {
                            let _ = handle.emit(
                                "script-execution",
                                ScriptExecutionEvent {
                                    execution_id: eid.clone(),
                                    script_id: sid.clone(),
                                    status: "completed".to_string(),
                                    message: Some("脚本执行完成".to_string()),
                                },
                            );
                        }
                    }
                    (true, None)
                }
                Err(e) => {
                    let is_terminated = match &*e {
                        rhai::EvalAltResult::ErrorTerminated(val, _) => {
                            val.to_string() == "脚本执行已停止"
                        }
                        _ => false,
                    };

                    if is_terminated {
                        tracing::info!(execution_id = %eid, script_id = %sid, "脚本执行被用户手动停止");
                        (true, None)
                    } else {
                        tracing::error!(execution_id = %eid, script_id = %sid, error = %e, "脚本执行出错");
                        {
                            let handle_guard = app_handle.lock();
                            if let Some(ref handle) = *handle_guard {
                                let _ = handle.emit(
                                    "script-execution",
                                    ScriptExecutionEvent {
                                        execution_id: eid.clone(),
                                        script_id: sid.clone(),
                                        status: "error".to_string(),
                                        message: Some(format!("脚本执行出错: {}", e)),
                                    },
                                );
                            }
                        }
                        (false, Some(e.to_string()))
                    }
                }
            };

            // 脚本执行结束（正常完成、报错或中断），自动重置所有受控手柄的状态，防止物理按键卡死
            controller.reset_all_devices();

            {
                let handle_guard = app_handle.lock();
                if let Some(ref handle) = *handle_guard {
                    let _ = handle.emit(
                        "script-line-change",
                        ScriptLineChangeEvent {
                            execution_id: eid.clone(),
                            script_id: sid.clone(),
                            line: 0,
                        },
                    );
                }
            }

            let mut executions = executions.lock();
            if let Some(exec) = executions.get_mut(&eid) {
                exec.running = false;
                exec.success = success;
                exec.error = err_msg;
            }
        });

        Ok(execution_id)
    }

    pub fn stop_execution(&self, execution_id: &str) -> Result<(), String> {
        let mut executions = self.executions.lock();
        if let Some(exec) = executions.get_mut(execution_id) {
            exec.token.cancel();
        }
        if executions.remove(execution_id).is_some() {
            tracing::info!(execution_id = %execution_id, "脚本执行已停止");
            Ok(())
        } else {
            Err(format!("执行不存在: {}", execution_id))
        }
    }

    pub fn is_executing(&self, execution_id: &str) -> bool {
        let executions = self.executions.lock();
        executions.get(execution_id).map_or(false, |e| e.running)
    }

    pub fn list_executions(&self) -> Vec<(String, bool)> {
        let executions = self.executions.lock();
        executions
            .iter()
            .map(|(id, e)| (id.clone(), e.running))
            .collect()
    }

    pub(super) fn wrap_script(code: &str) -> String {
        if code.contains("fn main()") || code.contains("fn main ()") {
            format!("{}\nmain();", code)
        } else {
            code.to_string()
        }
    }
}
