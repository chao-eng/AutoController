use parking_lot::Mutex;
use rhai::Engine;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

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
    success: bool,
    error: Option<String>,
}

pub struct ScriptRuntime {
    scripts: Arc<Mutex<HashMap<String, Script>>>,
    executions: Arc<Mutex<HashMap<String, Execution>>>,
    sequence_executions: Arc<Mutex<HashMap<String, SequenceExecution>>>,
    data_dir: Arc<DataDir>,
    controller: Arc<ControllerManager>,
    app_handle: Arc<Mutex<Option<AppHandle>>>,
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
            app_handle: Arc::new(Mutex::new(None)),
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
            app_handle: Arc::new(Mutex::new(None)),
        }
    }

    pub fn set_app_handle(&self, handle: AppHandle) {
        let mut app_handle = self.app_handle.lock();
        *app_handle = Some(handle);
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
            executions.insert(execution_id.clone(), Execution {
                running: true,
                success: false,
                error: None,
            });
        }

        let controller = self.controller.clone();
        let executions = self.executions.clone();
        let app_handle = self.app_handle.clone();

        {
            let app_handle_guard = app_handle.lock();
            if let Some(ref handle) = *app_handle_guard {
                let _ = handle.emit("script-execution", ScriptExecutionEvent {
                    execution_id: execution_id.clone(),
                    script_id: script_id.to_string(),
                    status: "started".to_string(),
                    message: Some(format!("脚本 '{}' 开始执行", name)),
                });
            }
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
                {
                    let handle_guard = handle_press.lock();
                    if let Some(ref handle) = *handle_guard {
                        if let Some(line) = context.call_position().line() {
                            let _ = handle.emit("script-line-change", ScriptLineChangeEvent {
                                execution_id: eid_press.clone(),
                                script_id: sid_press.clone(),
                                line,
                            });
                        }
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
                {
                    let handle_guard = handle_release.lock();
                    if let Some(ref handle) = *handle_guard {
                        if let Some(line) = context.call_position().line() {
                            let _ = handle.emit("script-line-change", ScriptLineChangeEvent {
                                execution_id: eid_release.clone(),
                                script_id: sid_release.clone(),
                                line,
                            });
                        }
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
                {
                    let handle_guard = handle_thumb_f.lock();
                    if let Some(ref handle) = *handle_guard {
                        if let Some(line) = context.call_position().line() {
                            let _ = handle.emit("script-line-change", ScriptLineChangeEvent {
                                execution_id: eid_thumb_f.clone(),
                                script_id: sid_thumb_f.clone(),
                                line,
                            });
                        }
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
                {
                    let handle_guard = handle_thumb_i.lock();
                    if let Some(ref handle) = *handle_guard {
                        if let Some(line) = context.call_position().line() {
                            let _ = handle.emit("script-line-change", ScriptLineChangeEvent {
                                execution_id: eid_thumb_i.clone(),
                                script_id: sid_thumb_i.clone(),
                                line,
                            });
                        }
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
                {
                    let handle_guard = handle_trig_f.lock();
                    if let Some(ref handle) = *handle_guard {
                        if let Some(line) = context.call_position().line() {
                            let _ = handle.emit("script-line-change", ScriptLineChangeEvent {
                                execution_id: eid_trig_f.clone(),
                                script_id: sid_trig_f.clone(),
                                line,
                            });
                        }
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
                {
                    let handle_guard = handle_trig_i.lock();
                    if let Some(ref handle) = *handle_guard {
                        if let Some(line) = context.call_position().line() {
                            let _ = handle.emit("script-line-change", ScriptLineChangeEvent {
                                execution_id: eid_trig_i.clone(),
                                script_id: sid_trig_i.clone(),
                                line,
                            });
                        }
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
                {
                    let handle_guard = handle_sleep.lock();
                    if let Some(ref handle) = *handle_guard {
                        if let Some(line) = context.call_position().line() {
                            let _ = handle.emit("script-line-change", ScriptLineChangeEvent {
                                execution_id: eid_sleep.clone(),
                                script_id: sid_sleep.clone(),
                                line,
                            });
                        }
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
                {
                    let handle_guard = handle_log.lock();
                    if let Some(ref handle) = *handle_guard {
                        if let Some(line) = context.call_position().line() {
                            let _ = handle.emit("script-line-change", ScriptLineChangeEvent {
                                execution_id: eid_log.clone(),
                                script_id: sid_log.clone(),
                                line,
                            });
                        }
                    }
                }
                tracing::info!(target: "script", "[脚本] {}", msg);
            });

            let handle_ocr_params = app_handle.clone();
            let eid_ocr_params = eid.clone();
            let sid_ocr_params = sid.clone();
            engine.register_fn("ocr", move |context: rhai::NativeCallContext, x: i64, y: i64, w: i64, h: i64| -> String {
                {
                    let handle_guard = handle_ocr_params.lock();
                    if let Some(ref handle) = *handle_guard {
                        if let Some(line) = context.call_position().line() {
                            let _ = handle.emit("script-line-change", ScriptLineChangeEvent {
                                execution_id: eid_ocr_params.clone(),
                                script_id: sid_ocr_params.clone(),
                                line,
                            });
                        }
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
                {
                    let handle_guard = handle_ocr_def.lock();
                    if let Some(ref handle) = *handle_guard {
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
                        
                        // 优先使用 ocr_regions 的第一个作为默认识别区，其次使用老字段 ocr_region 兼容
                        let target_region = if !config.ocr_regions.is_empty() {
                            Some(config.ocr_regions[0].clone())
                        } else {
                            config.ocr_region.clone()
                        };

                        if let Some(region) = target_region {
                            return match crate::script_engine::ocr::ocr_region_sync(region.x, region.y, region.w, region.h) {
                                Ok(text) => text,
                                Err(e) => {
                                    tracing::error!(target: "script", "OCR 默认区域 #1 识别出错: {}", e);
                                    String::new()
                                }
                            };
                        } else {
                            tracing::warn!(target: "script", "OCR 默认区域 #1 尚未配置，请在前端配置或传入坐标");
                            return String::new();
                        }
                    }
                }
                tracing::warn!(target: "script", "ocr() 无参调用失败：AppHandle 尚未初始化");
                String::new()
            });

            let handle_ocr_idx = app_handle.clone();
            let eid_ocr_idx = eid.clone();
            let sid_ocr_idx = sid.clone();
            engine.register_fn("ocr", move |context: rhai::NativeCallContext, index: i64| -> String {
                {
                    let handle_guard = handle_ocr_idx.lock();
                    if let Some(ref handle) = *handle_guard {
                        if let Some(line) = context.call_position().line() {
                            let _ = handle.emit("script-line-change", ScriptLineChangeEvent {
                                execution_id: eid_ocr_idx.clone(),
                                script_id: sid_ocr_idx.clone(),
                                line,
                            });
                        }
                        if index <= 0 {
                            tracing::error!(target: "script", "ocr(index) 序号错误：序号必须从 1 开始（传入为 {}）", index);
                            return String::new();
                        }
                        use tauri::Manager;
                        let config_mgr = handle.state::<crate::config::AppConfigManager>();
                        let config = config_mgr.get();
                        let u_idx = (index - 1) as usize;
                        
                        if u_idx < config.ocr_regions.len() {
                            let region = &config.ocr_regions[u_idx];
                            return match crate::script_engine::ocr::ocr_region_sync(region.x, region.y, region.w, region.h) {
                                Ok(text) => text,
                                Err(e) => {
                                    tracing::error!(target: "script", "OCR 区域 #{} 识别出错: {}", index, e);
                                    String::new()
                                }
                            };
                        } else {
                            tracing::warn!(target: "script", "OCR 区域 #{} 尚未配置，当前已配置区域数: {}", index, config.ocr_regions.len());
                            return String::new();
                        }
                    }
                }
                tracing::warn!(target: "script", "ocr(index) 调用失败：AppHandle 尚未初始化");
                String::new()
            });

            let wrapped_code = Self::wrap_script(&code);

            let (success, err_msg) = match engine.eval::<()>(&wrapped_code) {
                Ok(()) => {
                    tracing::info!(execution_id = %eid, script_id = %sid, "脚本执行完成");
                    {
                        let handle_guard = app_handle.lock();
                        if let Some(ref handle) = *handle_guard {
                            let _ = handle.emit("script-execution", ScriptExecutionEvent {
                                execution_id: eid.clone(),
                                script_id: sid.clone(),
                                status: "completed".to_string(),
                                message: Some("脚本执行完成".to_string()),
                            });
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
                                let _ = handle.emit("script-execution", ScriptExecutionEvent {
                                    execution_id: eid.clone(),
                                    script_id: sid.clone(),
                                    status: "error".to_string(),
                                    message: Some(format!("脚本执行出错: {}", e)),
                                });
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
                    let _ = handle.emit("script-line-change", ScriptLineChangeEvent {
                        execution_id: eid.clone(),
                        script_id: sid.clone(),
                        line: 0,
                    });
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
            let mut sequence_error: Option<String> = None;

            // Define overall loops (if 0 or 1, run once)
            let loops = if total_task_loops == 0 { 1 } else { total_task_loops };

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
                                executions.get(&exec_id)
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
                    let task_name = if let Some(queue) = handle.try_state::<crate::scheduler::TaskQueue>() {
                        queue.get_task(&task_id_str).map(|t| t.name.clone()).unwrap_or_else(|| "未知任务".to_string())
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

                    crate::notify::trigger_task_notification(handle, &task_id_str, &task_name, status, msg);
                }
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
