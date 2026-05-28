use parking_lot::Mutex;
use rhai::Engine;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

use super::types::*;
use crate::controller::ControllerManager;
use crate::persistence::DataDir;
use chrono::Utc;

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

struct SequenceExecution {
    running: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
struct ScriptExecutionEvent {
    execution_id: String,
    script_id: String,
    status: String,
    message: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct ScriptLineChangeEvent {
    execution_id: String,
    script_id: String,
    line: usize,
}

struct Execution {
    running: bool,
}

pub struct ScriptRuntime {
    scripts: Arc<Mutex<HashMap<String, Script>>>,
    executions: Arc<Mutex<HashMap<String, Execution>>>,
    sequence_executions: Arc<Mutex<HashMap<String, SequenceExecution>>>,
    data_dir: Arc<DataDir>,
    controller: Arc<ControllerManager>,
    app_handle: Option<AppHandle>,
}

unsafe impl Send for ScriptRuntime {}
unsafe impl Sync for ScriptRuntime {}

impl Clone for ScriptRuntime {
    fn clone(&self) -> Self {
        Self {
            scripts: self.scripts.clone(),
            executions: self.executions.clone(),
            sequence_executions: self.sequence_executions.clone(),
            data_dir: self.data_dir.clone(),
            controller: self.controller.clone(),
            app_handle: self.app_handle.clone(),
        }
    }
}

impl ScriptRuntime {
    pub fn new() -> Self {
        let data_dir = Arc::new(DataDir::new());
        let scripts = match data_dir.load::<HashMap<String, Script>>("scripts") {
            Some(data) => Arc::new(Mutex::new(data)),
            None => Arc::new(Mutex::new(HashMap::new())),
        };
        let controller = Arc::new(ControllerManager::new());

        Self {
            scripts,
            executions: Arc::new(Mutex::new(HashMap::new())),
            sequence_executions: Arc::new(Mutex::new(HashMap::new())),
            data_dir,
            controller,
            app_handle: None,
        }
    }

    pub fn with_controller(controller: ControllerManager) -> Self {
        let data_dir = Arc::new(DataDir::new());
        let scripts = match data_dir.load::<HashMap<String, Script>>("scripts") {
            Some(data) => Arc::new(Mutex::new(data)),
            None => Arc::new(Mutex::new(HashMap::new())),
        };

        Self {
            scripts,
            executions: Arc::new(Mutex::new(HashMap::new())),
            sequence_executions: Arc::new(Mutex::new(HashMap::new())),
            data_dir,
            controller: Arc::new(controller),
            app_handle: None,
        }
    }

    pub fn set_app_handle(&self, handle: AppHandle) {
        let self_ptr = self as *const Self as *mut Self;
        unsafe {
            (*self_ptr).app_handle = Some(handle);
        }
    }

    fn persist(&self) {
        let scripts = self.scripts.lock();
        if let Err(e) = self.data_dir.save("scripts", &*scripts) {
            tracing::warn!(error = %e, "脚本数据持久化失败");
        }
    }

    pub fn create_script(&self, name: &str, code: &str) -> Script {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();
        let script = Script {
            id: id.clone(),
            name: name.to_string(),
            code: code.to_string(),
            created_at: now,
            updated_at: now,
        };
        let mut scripts = self.scripts.lock();
        scripts.insert(id, script.clone());
        drop(scripts);
        self.persist();
        tracing::info!(script_id = %script.id, name, "脚本已创建");
        script
    }

    pub fn update_script(&self, id: &str, code: &str) -> Result<Script, String> {
        let mut scripts = self.scripts.lock();
        if let Some(script) = scripts.get_mut(id) {
            script.code = code.to_string();
            script.updated_at = Utc::now();
            let updated = script.clone();
            drop(scripts);
            self.persist();
            Ok(updated)
        } else {
            Err(format!("脚本不存在: {}", id))
        }
    }

    pub fn rename_script(&self, id: &str, new_name: &str) -> Result<Script, String> {
        let mut scripts = self.scripts.lock();
        if let Some(script) = scripts.get_mut(id) {
            script.name = new_name.to_string();
            script.updated_at = Utc::now();
            let updated = script.clone();
            drop(scripts);
            self.persist();
            Ok(updated)
        } else {
            Err(format!("脚本不存在: {}", id))
        }
    }


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

        {
            let mut executions = self.executions.lock();
            executions.insert(execution_id.clone(), Execution { running: true });
        }

        let controller = self.controller.clone();
        let executions = self.executions.clone();
        let app_handle = self.app_handle.clone();

        if let Some(ref handle) = app_handle {
            let _ = handle.emit("script-execution", ScriptExecutionEvent {
                execution_id: execution_id.clone(),
                script_id: script_id.to_string(),
                status: "started".to_string(),
                message: Some(format!("脚本 '{}' 开始执行", name)),
            });
        }

        std::thread::spawn(move || {
            tracing::info!(execution_id = %eid, script_id = %sid, name = %name, "脚本开始执行");

            let mut engine = Engine::new();
            engine.set_allow_looping(true);
            engine.set_max_operations(10_000_000);
            engine.set_max_string_size(100_000);
            engine.set_max_array_size(10_000);

            let default_device = Arc::new(Mutex::new("0".to_string()));

            // set_default_device(device_id)
            let def_device = default_device.clone();
            engine.register_fn("set_default_device", move |device_id: i64| {
                let mut d = def_device.lock();
                *d = device_id.to_string();
            });

            let def_device = default_device.clone();
            engine.register_fn("set_default_device", move |device_id: &str| {
                let mut d = def_device.lock();
                *d = device_id.to_string();
            });

            // press(device_id, button) & press(button)
            let ctrl = controller.clone();
            engine.register_fn("press", move |device_id: i64, btn: &str| {
                if let Some(b) = parse_button(btn) {
                    if let Err(e) = ctrl.set_button(&device_id.to_string(), b, true) {
                        tracing::warn!(target: "script", error = %e, "press 失败");
                    }
                } else {
                    tracing::warn!(target: "script", "未知按键: {}", btn);
                }
            });

            let ctrl = controller.clone();
            engine.register_fn("press", move |device_id: &str, btn: &str| {
                if let Some(b) = parse_button(btn) {
                    if let Err(e) = ctrl.set_button(device_id, b, true) {
                        tracing::warn!(target: "script", error = %e, "press 失败");
                    }
                } else {
                    tracing::warn!(target: "script", "未知按键: {}", btn);
                }
            });

            let ctrl = controller.clone();
            let def_device = default_device.clone();
            let handle_press = app_handle.clone();
            let eid_press = eid.clone();
            let sid_press = sid.clone();
            engine.register_fn("press", move |context: rhai::NativeCallContext, btn: &str| {
                if let Some(ref handle) = handle_press {
                    if let Some(line) = context.call_position().line() {
                        let _ = handle.emit("script-line-change", ScriptLineChangeEvent {
                            execution_id: eid_press.clone(),
                            script_id: sid_press.clone(),
                            line,
                        });
                    }
                }
                let dev = def_device.lock().clone();
                if let Some(b) = parse_button(btn) {
                    if let Err(e) = ctrl.set_button(&dev, b, true) {
                        tracing::warn!(target: "script", error = %e, "press 失败");
                    }
                } else {
                    tracing::warn!(target: "script", "未知按键: {}", btn);
                }
            });

            // release(device_id, button) & release(button)
            let ctrl = controller.clone();
            engine.register_fn("release", move |device_id: i64, btn: &str| {
                if let Some(b) = parse_button(btn) {
                    if let Err(e) = ctrl.set_button(&device_id.to_string(), b, false) {
                        tracing::warn!(target: "script", error = %e, "release 失败");
                    }
                } else {
                    tracing::warn!(target: "script", "未知按键: {}", btn);
                }
            });

            let ctrl = controller.clone();
            engine.register_fn("release", move |device_id: &str, btn: &str| {
                if let Some(b) = parse_button(btn) {
                    if let Err(e) = ctrl.set_button(device_id, b, false) {
                        tracing::warn!(target: "script", error = %e, "release 失败");
                    }
                } else {
                    tracing::warn!(target: "script", "未知按键: {}", btn);
                }
            });

            let ctrl = controller.clone();
            let def_device = default_device.clone();
            let handle_release = app_handle.clone();
            let eid_release = eid.clone();
            let sid_release = sid.clone();
            engine.register_fn("release", move |context: rhai::NativeCallContext, btn: &str| {
                if let Some(ref handle) = handle_release {
                    if let Some(line) = context.call_position().line() {
                        let _ = handle.emit("script-line-change", ScriptLineChangeEvent {
                            execution_id: eid_release.clone(),
                            script_id: sid_release.clone(),
                            line,
                        });
                    }
                }
                let dev = def_device.lock().clone();
                if let Some(b) = parse_button(btn) {
                    if let Err(e) = ctrl.set_button(&dev, b, false) {
                        tracing::warn!(target: "script", error = %e, "release 失败");
                    }
                } else {
                    tracing::warn!(target: "script", "未知按键: {}", btn);
                }
            });

            // set_thumb(device_id, axis, value) & set_thumb(axis, value)
            let ctrl = controller.clone();
            engine.register_fn("set_thumb", move |device_id: i64, axis: &str, val: f64| {
                if let Some(a) = parse_axis(axis) {
                    if let Err(e) = ctrl.set_thumb(&device_id.to_string(), a, val as f32) {
                        tracing::warn!(target: "script", error = %e, "set_thumb 失败");
                    }
                } else {
                    tracing::warn!(target: "script", "未知摇杆轴: {}", axis);
                }
            });

            let ctrl = controller.clone();
            engine.register_fn("set_thumb", move |device_id: i64, axis: &str, val: i64| {
                if let Some(a) = parse_axis(axis) {
                    if let Err(e) = ctrl.set_thumb(&device_id.to_string(), a, val as f32) {
                        tracing::warn!(target: "script", error = %e, "set_thumb 失败");
                    }
                } else {
                    tracing::warn!(target: "script", "未知摇杆轴: {}", axis);
                }
            });

            let ctrl = controller.clone();
            engine.register_fn("set_thumb", move |device_id: &str, axis: &str, val: f64| {
                if let Some(a) = parse_axis(axis) {
                    if let Err(e) = ctrl.set_thumb(device_id, a, val as f32) {
                        tracing::warn!(target: "script", error = %e, "set_thumb 失败");
                    }
                } else {
                    tracing::warn!(target: "script", "未知摇杆轴: {}", axis);
                }
            });

            let ctrl = controller.clone();
            engine.register_fn("set_thumb", move |device_id: &str, axis: &str, val: i64| {
                if let Some(a) = parse_axis(axis) {
                    if let Err(e) = ctrl.set_thumb(device_id, a, val as f32) {
                        tracing::warn!(target: "script", error = %e, "set_thumb 失败");
                    }
                } else {
                    tracing::warn!(target: "script", "未知摇杆轴: {}", axis);
                }
            });

            let ctrl = controller.clone();
            let def_device = default_device.clone();
            let handle_thumb_f = app_handle.clone();
            let eid_thumb_f = eid.clone();
            let sid_thumb_f = sid.clone();
            engine.register_fn("set_thumb", move |context: rhai::NativeCallContext, axis: &str, val: f64| {
                if let Some(ref handle) = handle_thumb_f {
                    if let Some(line) = context.call_position().line() {
                        let _ = handle.emit("script-line-change", ScriptLineChangeEvent {
                            execution_id: eid_thumb_f.clone(),
                            script_id: sid_thumb_f.clone(),
                            line,
                        });
                    }
                }
                let dev = def_device.lock().clone();
                if let Some(a) = parse_axis(axis) {
                    if let Err(e) = ctrl.set_thumb(&dev, a, val as f32) {
                        tracing::warn!(target: "script", error = %e, "set_thumb 失败");
                    }
                } else {
                    tracing::warn!(target: "script", "未知摇杆轴: {}", axis);
                }
            });

            let ctrl = controller.clone();
            let def_device = default_device.clone();
            let handle_thumb_i = app_handle.clone();
            let eid_thumb_i = eid.clone();
            let sid_thumb_i = sid.clone();
            engine.register_fn("set_thumb", move |context: rhai::NativeCallContext, axis: &str, val: i64| {
                if let Some(ref handle) = handle_thumb_i {
                    if let Some(line) = context.call_position().line() {
                        let _ = handle.emit("script-line-change", ScriptLineChangeEvent {
                            execution_id: eid_thumb_i.clone(),
                            script_id: sid_thumb_i.clone(),
                            line,
                        });
                    }
                }
                let dev = def_device.lock().clone();
                if let Some(a) = parse_axis(axis) {
                    if let Err(e) = ctrl.set_thumb(&dev, a, val as f32) {
                        tracing::warn!(target: "script", error = %e, "set_thumb 失败");
                    }
                } else {
                    tracing::warn!(target: "script", "未知摇杆轴: {}", axis);
                }
            });

            // set_trigger(device_id, side, value) & set_trigger(side, value)
            let ctrl = controller.clone();
            engine.register_fn("set_trigger", move |device_id: i64, side: &str, val: f64| {
                if let Some(s) = parse_trigger(side) {
                    if let Err(e) = ctrl.set_trigger(&device_id.to_string(), s, val as f32) {
                        tracing::warn!(target: "script", error = %e, "set_trigger 失败");
                    }
                } else {
                    tracing::warn!(target: "script", "未知扳机侧: {}", side);
                }
            });

            let ctrl = controller.clone();
            engine.register_fn("set_trigger", move |device_id: i64, side: &str, val: i64| {
                if let Some(s) = parse_trigger(side) {
                    if let Err(e) = ctrl.set_trigger(&device_id.to_string(), s, val as f32) {
                        tracing::warn!(target: "script", error = %e, "set_trigger 失败");
                    }
                } else {
                    tracing::warn!(target: "script", "未知扳机侧: {}", side);
                }
            });

            let ctrl = controller.clone();
            engine.register_fn("set_trigger", move |device_id: &str, side: &str, val: f64| {
                if let Some(s) = parse_trigger(side) {
                    if let Err(e) = ctrl.set_trigger(device_id, s, val as f32) {
                        tracing::warn!(target: "script", error = %e, "set_trigger 失败");
                    }
                } else {
                    tracing::warn!(target: "script", "未知扳机侧: {}", side);
                }
            });

            let ctrl = controller.clone();
            engine.register_fn("set_trigger", move |device_id: &str, side: &str, val: i64| {
                if let Some(s) = parse_trigger(side) {
                    if let Err(e) = ctrl.set_trigger(device_id, s, val as f32) {
                        tracing::warn!(target: "script", error = %e, "set_trigger 失败");
                    }
                } else {
                    tracing::warn!(target: "script", "未知扳机侧: {}", side);
                }
            });

            let ctrl = controller.clone();
            let def_device = default_device.clone();
            let handle_trig_f = app_handle.clone();
            let eid_trig_f = eid.clone();
            let sid_trig_f = sid.clone();
            engine.register_fn("set_trigger", move |context: rhai::NativeCallContext, side: &str, val: f64| {
                if let Some(ref handle) = handle_trig_f {
                    if let Some(line) = context.call_position().line() {
                        let _ = handle.emit("script-line-change", ScriptLineChangeEvent {
                            execution_id: eid_trig_f.clone(),
                            script_id: sid_trig_f.clone(),
                            line,
                        });
                    }
                }
                let dev = def_device.lock().clone();
                if let Some(s) = parse_trigger(side) {
                    if let Err(e) = ctrl.set_trigger(&dev, s, val as f32) {
                        tracing::warn!(target: "script", error = %e, "set_trigger 失败");
                    }
                } else {
                    tracing::warn!(target: "script", "未知扳机侧: {}", side);
                }
            });

            let ctrl = controller.clone();
            let def_device = default_device.clone();
            let handle_trig_i = app_handle.clone();
            let eid_trig_i = eid.clone();
            let sid_trig_i = sid.clone();
            engine.register_fn("set_trigger", move |context: rhai::NativeCallContext, side: &str, val: i64| {
                if let Some(ref handle) = handle_trig_i {
                    if let Some(line) = context.call_position().line() {
                        let _ = handle.emit("script-line-change", ScriptLineChangeEvent {
                            execution_id: eid_trig_i.clone(),
                            script_id: sid_trig_i.clone(),
                            line,
                        });
                    }
                }
                let dev = def_device.lock().clone();
                if let Some(s) = parse_trigger(side) {
                    if let Err(e) = ctrl.set_trigger(&dev, s, val as f32) {
                        tracing::warn!(target: "script", error = %e, "set_trigger 失败");
                    }
                } else {
                    tracing::warn!(target: "script", "未知扳机侧: {}", side);
                }
            });

            let executions_check = executions.clone();
            let check_id = eid.clone();
            engine.on_progress(move |_ops| {
                let executions = executions_check.lock();
                if !executions.contains_key(&check_id) {
                    Some(rhai::Dynamic::from("脚本执行已停止".to_string()))
                } else {
                    None
                }
            });

            let executions_sleep = executions.clone();
            let sleep_id = eid.clone();
            let handle_sleep = app_handle.clone();
            let eid_sleep = eid.clone();
            let sid_sleep = sid.clone();
            engine.register_fn("sleep", move |context: rhai::NativeCallContext, ms: i64| {
                if let Some(ref handle) = handle_sleep {
                    if let Some(line) = context.call_position().line() {
                        let _ = handle.emit("script-line-change", ScriptLineChangeEvent {
                            execution_id: eid_sleep.clone(),
                            script_id: sid_sleep.clone(),
                            line,
                        });
                    }
                }
                let total = ms as u64;
                let step = 50;
                let mut elapsed = 0;
                while elapsed < total {
                    {
                        let executions = executions_sleep.lock();
                        if !executions.contains_key(&sleep_id) {
                            break;
                        }
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
                if let Some(ref handle) = handle_log {
                    if let Some(line) = context.call_position().line() {
                        let _ = handle.emit("script-line-change", ScriptLineChangeEvent {
                            execution_id: eid_log.clone(),
                            script_id: sid_log.clone(),
                            line,
                        });
                    }
                }
                tracing::info!(target: "script", "[脚本] {}", msg);
            });

            let handle_ocr_params = app_handle.clone();
            let eid_ocr_params = eid.clone();
            let sid_ocr_params = sid.clone();
            engine.register_fn("ocr", move |context: rhai::NativeCallContext, x: i64, y: i64, w: i64, h: i64| -> String {
                if let Some(ref handle) = handle_ocr_params {
                    if let Some(line) = context.call_position().line() {
                        let _ = handle.emit("script-line-change", ScriptLineChangeEvent {
                            execution_id: eid_ocr_params.clone(),
                            script_id: sid_ocr_params.clone(),
                            line,
                        });
                    }
                }
                match crate::script_engine::ocr::ocr_region_sync(x as i32, y as i32, w as i32, h as i32) {
                    Ok(text) => text,
                    Err(e) => {
                        tracing::error!(target: "script", "OCR 识别出错: {}", e);
                        String::new()
                    }
                }
            });

            let handle_ocr_def = app_handle.clone();
            let eid_ocr_def = eid.clone();
            let sid_ocr_def = sid.clone();
            engine.register_fn("ocr", move |context: rhai::NativeCallContext| -> String {
                if let Some(ref handle) = handle_ocr_def {
                    if let Some(line) = context.call_position().line() {
                        let _ = handle.emit("script-line-change", ScriptLineChangeEvent {
                            execution_id: eid_ocr_def.clone(),
                            script_id: sid_ocr_def.clone(),
                            line,
                        });
                    }
                    use tauri::Manager;
                    let config_mgr = handle.state::<crate::config::AppConfigManager>();
                    let config = config_mgr.get();
                    if let Some(region) = config.ocr_region {
                        match crate::script_engine::ocr::ocr_region_sync(region.x, region.y, region.w, region.h) {
                            Ok(text) => text,
                            Err(e) => {
                                tracing::error!(target: "script", "OCR 默认区域识别出错: {}", e);
                                String::new()
                            }
                        }
                    } else {
                        tracing::warn!(target: "script", "OCR 默认区域尚未配置，请在前端配置或传入具体坐标参数");
                        String::new()
                    }
                } else {
                    tracing::warn!(target: "script", "ocr() 无参调用失败：AppHandle 尚未初始化");
                    String::new()
                }
            });

            let wrapped_code = Self::wrap_script(&code);

            match engine.eval::<()>(&wrapped_code) {
                Ok(()) => {
                    tracing::info!(execution_id = %eid, script_id = %sid, "脚本执行完成");
                    if let Some(ref handle) = app_handle {
                        let _ = handle.emit("script-execution", ScriptExecutionEvent {
                            execution_id: eid.clone(),
                            script_id: sid.clone(),
                            status: "completed".to_string(),
                            message: Some("脚本执行完成".to_string()),
                        });
                    }
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
                    } else {
                        tracing::error!(execution_id = %eid, script_id = %sid, error = %e, "脚本执行出错");
                        if let Some(ref handle) = app_handle {
                            let _ = handle.emit("script-execution", ScriptExecutionEvent {
                                execution_id: eid.clone(),
                                script_id: sid.clone(),
                                status: "error".to_string(),
                                message: Some(format!("脚本执行出错: {}", e)),
                            });
                        }
                    }
                }
            }

            // 脚本执行结束（正常完成、报错或中断），自动重置所有受控手柄的状态，防止物理按键卡死
            controller.reset_all_devices();

            if let Some(ref handle) = app_handle {
                let _ = handle.emit("script-line-change", ScriptLineChangeEvent {
                    execution_id: eid.clone(),
                    script_id: sid.clone(),
                    line: 0,
                });
            }

            let mut executions = executions.lock();
            if let Some(exec) = executions.get_mut(&eid) {
                exec.running = false;
            }
        });

        Ok(execution_id)
    }

    pub fn execute_sequence(
        &self,
        task_id: &str,
        steps: Vec<crate::scheduler::types::ScriptStep>,
        total_task_loops: u32,
    ) -> Result<(), String> {
        let eid = task_id.to_string();
        
        {
            let mut seq_execs = self.sequence_executions.lock();
            seq_execs.insert(eid.clone(), SequenceExecution { running: true });
        }

        let runtime = self.clone();
        let task_id_str = task_id.to_string();
        let app_handle = self.app_handle.clone();

        std::thread::spawn(move || {
            tracing::info!(task_id = %task_id_str, "开始顺序执行多脚本任务序列");
            
            let total_steps = steps.len();
            let mut cancelled = false;

            // Define overall loops (if 0 or 1, run once)
            let loops = if total_task_loops == 0 { 1 } else { total_task_loops };

            for task_loop in 1..=loops {
                if cancelled {
                    break;
                }

                for (step_idx, step) in steps.iter().enumerate() {
                    if cancelled {
                        break;
                    }

                    // Get script name
                    let script_name = {
                        let scripts = runtime.scripts.lock();
                        scripts.get(&step.script_id).map(|s| s.name.clone()).unwrap_or_else(|| "未知脚本".to_string())
                    };

                    let step_loops = if step.loop_count == 0 { 1 } else { step.loop_count };

                    for step_loop in 1..=step_loops {
                        // Check cancel signal
                        {
                            let seq_execs = runtime.sequence_executions.lock();
                            if !seq_execs.contains_key(&eid) {
                                cancelled = true;
                                break;
                            }
                        }

                        // Emit progress to frontend
                        if let Some(ref handle) = app_handle {
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

                        // Start single script execution
                        let exec_id = match runtime.execute_script(&step.script_id) {
                            Ok(id) => id,
                            Err(e) => {
                                tracing::error!(task_id = %task_id_str, error = %e, "步骤脚本启动失败");
                                break;
                            }
                        };

                        // Poll wait for script to finish
                        loop {
                            // Check cancel signal
                            {
                                let seq_execs = runtime.sequence_executions.lock();
                                if !seq_execs.contains_key(&eid) {
                                    cancelled = true;
                                    // Stop the running script execution
                                    let _ = runtime.stop_execution(&exec_id);
                                    break;
                                }
                            }

                            if !runtime.is_executing(&exec_id) {
                                break;
                            }

                            std::thread::sleep(std::time::Duration::from_millis(50));
                        }

                        if cancelled {
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

            if let Some(ref handle) = app_handle {
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
            }

            tracing::info!(task_id = %task_id_str, cancelled, "顺序执行任务序列已结束");
        });

        Ok(())
    }

    pub fn stop_sequence(&self, task_id: &str) -> Result<(), String> {
        let mut seq_execs = self.sequence_executions.lock();
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

    fn wrap_script(code: &str) -> String {
        if code.contains("fn main()") || code.contains("fn main ()") {
            format!("{}\nmain();", code)
        } else {
            code.to_string()
        }
    }

    pub fn stop_execution(&self, execution_id: &str) -> Result<(), String> {
        let mut executions = self.executions.lock();
        if executions.remove(execution_id).is_some() {
            tracing::info!(execution_id = %execution_id, "脚本执行已停止");
            Ok(())
        } else {
            Err(format!("执行不存在: {}", execution_id))
        }
    }

    pub fn list_scripts(&self) -> Vec<ScriptMeta> {
        let scripts = self.scripts.lock();
        scripts
            .values()
            .map(|s| ScriptMeta {
                id: s.id.clone(),
                name: s.name.clone(),
                created_at: s.created_at,
                updated_at: s.updated_at,
            })
            .collect()
    }

    pub fn get_script(&self, id: &str) -> Option<Script> {
        let scripts = self.scripts.lock();
        scripts.get(id).cloned()
    }

    pub fn delete_script(&self, id: &str) -> Result<(), String> {
        let mut scripts = self.scripts.lock();
        if scripts.remove(id).is_some() {
            drop(scripts);
            self.persist();
            Ok(())
        } else {
            Err(format!("脚本不存在: {}", id))
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
}

fn parse_button(btn: &str) -> Option<crate::controller::Button> {
    match btn.to_uppercase().as_str() {
        "A" => Some(crate::controller::Button::A),
        "B" => Some(crate::controller::Button::B),
        "X" => Some(crate::controller::Button::X),
        "Y" => Some(crate::controller::Button::Y),
        "LB" => Some(crate::controller::Button::LB),
        "RB" => Some(crate::controller::Button::RB),
        "LT" => Some(crate::controller::Button::LT),
        "RT" => Some(crate::controller::Button::RT),
        "BACK" => Some(crate::controller::Button::Back),
        "START" => Some(crate::controller::Button::Start),
        "GUIDE" => Some(crate::controller::Button::Guide),
        "LS" | "L3" => Some(crate::controller::Button::LeftThumb),
        "RS" | "R3" => Some(crate::controller::Button::RightThumb),
        "UP" | "DPAD_UP" => Some(crate::controller::Button::DPadUp),
        "DOWN" | "DPAD_DOWN" => Some(crate::controller::Button::DPadDown),
        "LEFT" | "DPAD_LEFT" => Some(crate::controller::Button::DPadLeft),
        "RIGHT" | "DPAD_RIGHT" => Some(crate::controller::Button::DPadRight),
        _ => None,
    }
}

fn parse_axis(axis: &str) -> Option<crate::controller::ThumbAxis> {
    match axis.to_lowercase().as_str() {
        "leftx" | "lx" => Some(crate::controller::ThumbAxis::LeftX),
        "lefty" | "ly" => Some(crate::controller::ThumbAxis::LeftY),
        "rightx" | "rx" => Some(crate::controller::ThumbAxis::RightX),
        "righty" | "ry" => Some(crate::controller::ThumbAxis::RightY),
        _ => None,
    }
}

fn parse_trigger(side: &str) -> Option<crate::controller::TriggerSide> {
    match side.to_lowercase().as_str() {
        "left" | "l" | "lt" => Some(crate::controller::TriggerSide::Left),
        "right" | "r" | "rt" => Some(crate::controller::TriggerSide::Right),
        _ => None,
    }
}
