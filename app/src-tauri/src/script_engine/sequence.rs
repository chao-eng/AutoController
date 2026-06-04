use tauri::{Emitter, Manager};


use super::runtime::ScriptRuntime;
use super::cancellation::CancellationToken;

#[derive(Debug, Clone, serde::Serialize)]
pub struct SequenceProgress {
    pub task_id: String,
    pub running: bool,
    pub current_task_loop: u32,
    pub total_task_loops: u32,
    pub current_step_index: usize,
    pub total_steps: usize,
    pub current_step_loop: u32,
    pub total_step_loops: u32,
    pub current_script_name: String,
}

pub(super) struct SequenceExecution {
    pub(super) running: bool,
    pub(super) token: CancellationToken,
}

impl ScriptRuntime {
    pub fn execute_sequence(
        &self,
        task_id: &str,
        steps: Vec<crate::scheduler::types::ScriptStep>,
        total_task_loops: u32,
    ) -> Result<(), String> {
        let eid = task_id.to_string();
        let token = CancellationToken::new();

        {
            let mut seq_execs = self.sequence_executions.lock();
            seq_execs.insert(
                eid.clone(),
                SequenceExecution {
                    running: true,
                    token: token.clone(),
                },
            );
        }

        let runtime = self.clone();
        let task_id_str = task_id.to_string();
        let app_handle = self.app_handle.clone();
        let token_thread = token.clone();

        std::thread::spawn(move || {
            tracing::info!(task_id = %task_id_str, "开始顺序执行多脚本任务序列");

            let total_steps = steps.len();
            let mut cancelled = false;
            let mut sequence_error: Option<String> = None;

            // Define overall loops (if 0 or 1, run once)
            let loops = if total_task_loops == 0 {
                1
            } else {
                total_task_loops
            };

            for task_loop in 1..=loops {
                if cancelled || sequence_error.is_some() {
                    break;
                }

                for (step_idx, step) in steps.iter().enumerate() {
                    if cancelled || sequence_error.is_some() {
                        break;
                    }

                    // Get script name
                    let script_name = {
                        let scripts = runtime.scripts.lock();
                        scripts
                            .get(&step.script_id)
                            .map(|s| s.name.clone())
                            .unwrap_or_else(|| "未知脚本".to_string())
                    };

                    let step_loops = if step.loop_count == 0 {
                        1
                    } else {
                        step.loop_count
                    };

                    for step_loop in 1..=step_loops {
                        // Check cancel signal
                        if token_thread.is_cancelled() {
                            cancelled = true;
                            break;
                        }

                        // Emit progress to frontend
                        {
                            let handle_guard = app_handle.lock();
                            if let Some(ref handle) = *handle_guard {
                                let progress = SequenceProgress {
                                    task_id: task_id_str.clone(),
                                    running: true,
                                    current_task_loop: task_loop,
                                    total_task_loops: loops,
                                    current_step_index: step_idx,
                                    total_steps,
                                    current_step_loop: step_loop,
                                    total_step_loops: step_loops,
                                    current_script_name: script_name.clone(),
                                };
                                let _ = handle.emit("sequence-execution-progress", &progress);
                            }
                        }

                        // Start single script execution
                        let exec_id = match runtime.execute_script(&step.script_id) {
                            Ok(id) => id,
                            Err(e) => {
                                tracing::error!(task_id = %task_id_str, error = %e, "步骤脚本启动失败");
                                sequence_error = Some(format!("脚本启动失败: {}", e));
                                break;
                            }
                        };

                        // Poll wait for script to finish
                        loop {
                            // Check cancel signal
                            if token_thread.is_cancelled() {
                                cancelled = true;
                                // Stop the running script execution
                                let _ = runtime.stop_execution(&exec_id);
                                break;
                            }

                            if !runtime.is_executing(&exec_id) {
                                break;
                            }

                            std::thread::sleep(std::time::Duration::from_millis(50));
                        }

                        if cancelled {
                            break;
                        }

                        // Check if the script execution succeeded
                        let script_failed = {
                            let executions = runtime.executions.lock();
                            if let Some(exec) = executions.get(&exec_id) {
                                !exec.success
                            } else {
                                false
                            }
                        };

                        if script_failed {
                            let err_msg = {
                                let executions = runtime.executions.lock();
                                executions
                                    .get(&exec_id)
                                    .and_then(|exec| exec.error.clone())
                                    .unwrap_or_else(|| "步骤脚本执行出错".to_string())
                            };
                            sequence_error = Some(err_msg);
                            break;
                        }
                    }
                }
            }

            // Cleanup & final notification
            {
                let mut seq_execs = runtime.sequence_executions.lock();
                seq_execs.remove(&eid);
            }

            // Ensure virtual gamepad is reset
            runtime.controller.reset_all_devices();

            {
                let handle_guard = app_handle.lock();
                if let Some(ref handle) = *handle_guard {
                    let progress = SequenceProgress {
                        task_id: task_id_str.clone(),
                        running: false,
                        current_task_loop: 0,
                        total_task_loops: 0,
                        current_step_index: 0,
                        total_steps: 0,
                        current_step_loop: 0,
                        total_step_loops: 0,
                        current_script_name: String::new(),
                    };
                    let _ = handle.emit("sequence-execution-progress", &progress);

                    // 异步触发通知
                    let task_name =
                        if let Some(queue) = handle.try_state::<crate::scheduler::TaskQueue>() {
                            queue
                                .get_task(&task_id_str)
                                .map(|t| t.name.clone())
                                .unwrap_or_else(|| "未知任务".to_string())
                        } else {
                            "未知任务".to_string()
                        };

                    let (status, msg) = if let Some(ref err) = sequence_error {
                        ("interrupted", err.as_str())
                    } else if cancelled {
                        ("interrupted", "任务序列在执行过程中被用户手动停止或中断")
                    } else {
                        ("completed", "任务序列已成功执行完毕所有步骤与循环！")
                    };

                    crate::notify::trigger_task_notification(
                        handle,
                        &task_id_str,
                        &task_name,
                        status,
                        msg,
                    );
                }
            }

            tracing::info!(task_id = %task_id_str, cancelled, "顺序执行任务序列已结束");
        });

        Ok(())
    }

    pub fn stop_sequence(&self, task_id: &str) -> Result<(), String> {
        let mut seq_execs = self.sequence_executions.lock();
        if let Some(seq) = seq_execs.get_mut(task_id) {
            seq.token.cancel();
        }
        if seq_execs.remove(task_id).is_some() {
            tracing::info!(task_id = %task_id, "顺序执行任务序列已被手动中止");
            Ok(())
        } else {
            Err(format!("任务序列执行不存在: {}", task_id))
        }
    }

    pub fn is_sequence_executing(&self, task_id: &str) -> bool {
        let seq_execs = self.sequence_executions.lock();
        seq_execs.contains_key(task_id)
    }
}
