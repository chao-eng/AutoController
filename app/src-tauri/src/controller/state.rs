use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

use super::types::*;
use super::vigem::{ViGEmBindings, ViGEmClient, PVIGEM_TARGET};
use crate::persistence::DataDir;
use crate::macro_engine::{MacroRecorder, MacroEventType};
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, serde::Serialize)]
struct ControllerStateEvent {
    device_id: String,
    state: ControllerState,
}

struct ManagedDevice {
    info: DeviceInfo,
    target: Option<PVIGEM_TARGET>,
}

pub struct ControllerManager {
    devices: Arc<Mutex<HashMap<String, ManagedDevice>>>,
    next_index: Arc<Mutex<usize>>,
    vigem: Arc<Mutex<Option<ViGEmClient>>>,
    vigem_bindings: Arc<Mutex<Option<Arc<ViGEmBindings>>>>,
    vigem_error_code: Arc<Mutex<Option<u32>>>,
    data_dir: Arc<DataDir>,
    app_handle: Arc<Mutex<Option<AppHandle>>>,
    macro_recorder: MacroRecorder,
}

unsafe impl Send for ControllerManager {}
unsafe impl Sync for ControllerManager {}

impl Clone for ControllerManager {
    fn clone(&self) -> Self {
        Self {
            devices: self.devices.clone(),
            next_index: self.next_index.clone(),
            vigem: self.vigem.clone(),
            vigem_bindings: self.vigem_bindings.clone(),
            vigem_error_code: self.vigem_error_code.clone(),
            data_dir: self.data_dir.clone(),
            app_handle: self.app_handle.clone(),
            macro_recorder: self.macro_recorder.clone(),
        }
    }
}

impl ControllerManager {
    pub fn new() -> Self {
        let data_dir = Arc::new(DataDir::new());

        let (vigem_bindings, vigem_client, vigem_error_code) = match ViGEmBindings::load() {
            Ok(bindings) => {
                let bindings_arc = Arc::new(bindings);
                match ViGEmClient::new(bindings_arc.clone()) {
                    Ok(client) => {
                        tracing::info!("ViGEmBus 后端已启用，虚拟手柄将被系统识别");
                        (Some(bindings_arc), Some(client), None)
                    }
                    Err((code, msg)) => {
                        tracing::warn!(error = %msg, error_code = code, "ViGEmBus 连接失败，将使用模拟模式");
                        (Some(bindings_arc), None, Some(code))
                    }
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "ViGEmClient.dll 未找到，将使用模拟模式");
                (None, None, None)
            }
        };

        let macro_recorder = MacroRecorder::new();

        let manager = Self {
            devices: Arc::new(Mutex::new(HashMap::new())),
            next_index: Arc::new(Mutex::new(0)),
            vigem: Arc::new(Mutex::new(vigem_client)),
            vigem_bindings: Arc::new(Mutex::new(vigem_bindings)),
            vigem_error_code: Arc::new(Mutex::new(vigem_error_code)),
            data_dir,
            app_handle: Arc::new(Mutex::new(None)),
            macro_recorder,
        };

        manager.restore_devices();
        manager
    }

    pub fn macro_recorder(&self) -> MacroRecorder {
        self.macro_recorder.clone()
    }

    pub fn set_app_handle(&self, handle: AppHandle) {
        let mut app_handle = self.app_handle.lock();
        *app_handle = Some(handle);
    }

    fn emit_state_changed(&self, device_id: &str, state: &ControllerState) {
        let app_handle = self.app_handle.lock();
        if let Some(ref handle) = *app_handle {
            let _ = handle.emit(
                "controller-state-changed",
                ControllerStateEvent {
                    device_id: device_id.to_string(),
                    state: state.clone(),
                },
            );
        }
    }

    fn restore_devices(&self) {
        let configs = match self.data_dir.load::<Vec<DeviceConfig>>("devices") {
            Some(c) => c,
            None => return,
        };

        if configs.is_empty() {
            return;
        }

        tracing::info!(count = configs.len(), "正在恢复持久化的设备配置");

        let mut next_index = self.next_index.lock();
        let mut devices = self.devices.lock();
        let vigem = self.vigem.lock();

        for config in configs {
            let target = if config.connected {
                if let Some(ref client) = *vigem {
                    match config.controller_type {
                        ControllerType::Xbox360 => client.create_x360_target().ok(),
                        ControllerType::DualShock4 => client.create_ds4_target().ok(),
                    }
                } else {
                    None
                }
            } else {
                None
            };

            let vigem_connected = target.is_some();
            let device = ManagedDevice {
                info: DeviceInfo {
                    id: config.id.clone(),
                    index: config.index,
                    controller_type: config.controller_type,
                    connected: config.connected,
                    state: ControllerState::default(),
                    vigem_connected,
                },
                target,
            };

            if config.index >= *next_index {
                *next_index = config.index + 1;
            }

            devices.insert(config.id, device);
        }

        tracing::info!(count = devices.len(), "设备配置已恢复");
    }

    fn persist(&self) {
        let devices = self.devices.lock();
        let configs: Vec<DeviceConfig> = devices
            .values()
            .map(|d| DeviceConfig {
                id: d.info.id.clone(),
                index: d.info.index,
                controller_type: d.info.controller_type,
                connected: d.info.connected,
            })
            .collect();
        drop(devices);

        if let Err(e) = self.data_dir.save("devices", &configs) {
            tracing::warn!(error = %e, "设备配置持久化失败");
        }
    }

    pub fn create_device(&self, controller_type: ControllerType) -> Result<DeviceInfo, String> {
        let mut devices = self.devices.lock();
        let mut next_index = self.next_index.lock();

        if devices.len() >= 8 {
            return Err("已达到最大设备数量(8个)".to_string());
        }

        let id = uuid::Uuid::new_v4().to_string();
        let index = *next_index;
        *next_index += 1;

        let target = {
            let vigem = self.vigem.lock();
            if let Some(ref client) = *vigem {
                match controller_type {
                    ControllerType::Xbox360 => client.create_x360_target().ok(),
                    ControllerType::DualShock4 => client.create_ds4_target().ok(),
                }
            } else {
                None
            }
        };

        let vigem_connected = target.is_some();
        let device = ManagedDevice {
            info: DeviceInfo {
                id: id.clone(),
                index,
                controller_type,
                connected: true,
                state: ControllerState::default(),
                vigem_connected,
            },
            target,
        };

        let info = device.info.clone();
        devices.insert(id.clone(), device);
        drop(devices);

        self.persist();

        if vigem_connected {
            tracing::info!(
                device_id = %id,
                index = index,
                controller_type = %controller_type,
                "虚拟手柄已创建并连接到 ViGEmBus（系统可识别）"
            );
        } else {
            tracing::info!(
                device_id = %id,
                index = index,
                controller_type = %controller_type,
                "虚拟手柄已创建（模拟模式，ViGEmBus 不可用）"
            );
        }

        Ok(info)
    }

    pub fn remove_device(&self, device_id: &str) -> Result<(), String> {
        let mut devices = self.devices.lock();
        if let Some(mut device) = devices.remove(device_id) {
            if let Some(target) = device.target.take() {
                let vigem = self.vigem.lock();
                if let Some(ref client) = *vigem {
                    if let Err(e) = client.remove_target(target) {
                        tracing::warn!(error = %e, "ViGEmBus 设备移除失败");
                    }
                }
            }
            drop(devices);
            self.persist();
            tracing::info!(device_id = %device_id, "虚拟手柄设备已移除");
            Ok(())
        } else {
            Err(format!("设备不存在: {}", device_id))
        }
    }

    pub fn toggle_device_connection(&self, device_id: &str) -> Result<DeviceInfo, String> {
        let resolved_id = self.resolve_device_id(device_id)?;
        let mut devices = self.devices.lock();
        if let Some(device) = devices.get_mut(&resolved_id) {
            if device.info.connected {
                // Disconnect (plug out)
                if let Some(target) = device.target.take() {
                    let vigem = self.vigem.lock();
                    if let Some(ref client) = *vigem {
                        if let Err(e) = client.remove_target(target) {
                            tracing::warn!(error = %e, "ViGEmBus 设备断开失败");
                        }
                    }
                }
                device.info.connected = false;
                device.info.vigem_connected = false;
                tracing::info!(device_id = %resolved_id, "虚拟手柄已断开连接（保留在列表）");
            } else {
                // Connect (plug in)
                let target = {
                    let vigem = self.vigem.lock();
                    if let Some(ref client) = *vigem {
                        match device.info.controller_type {
                            ControllerType::Xbox360 => client.create_x360_target().ok(),
                            ControllerType::DualShock4 => client.create_ds4_target().ok(),
                        }
                    } else {
                        None
                    }
                };
                device.info.vigem_connected = target.is_some();
                device.target = target;
                device.info.connected = true;
                if device.info.vigem_connected {
                    tracing::info!(device_id = %resolved_id, "虚拟手柄已重新连接并挂载（系统可识别）");
                } else {
                    tracing::info!(device_id = %resolved_id, "虚拟手柄已重新连接（模拟模式，系统不可识别）");
                }
            }
            let info = device.info.clone();
            let state = device.info.state.clone();
            drop(devices);
            self.persist();
            self.emit_state_changed(&resolved_id, &state);
            Ok(info)
        } else {
            Err(format!("设备不存在: {}", resolved_id))
        }
    }

    fn resolve_device_id(&self, device_id: &str) -> Result<String, String> {
        if let Ok(index) = device_id.parse::<usize>() {
            let devices = self.devices.lock();
            let found = devices
                .values()
                .find(|d| d.info.index == index)
                .map(|d| d.info.id.clone());
            drop(devices);
            found.ok_or_else(|| format!("设备编号 {} 不存在", index))
        } else {
            let devices = self.devices.lock();
            if devices.contains_key(device_id) {
                Ok(device_id.to_string())
            } else {
                Err(format!("设备不存在: {}", device_id))
            }
        }
    }

    fn submit_report(&self, device: &ManagedDevice) {
        if let Some(target) = device.target {
            let vigem = self.vigem.lock();
            if let Some(ref client) = *vigem {
                let report = crate::controller::vigem::XusbReport::from(&device.info.state);
                if let Err(e) = client.update_x360(target, report) {
                    tracing::warn!(error = %e, "ViGEmBus 报告提交失败");
                }
            }
        }
    }

    pub fn set_button(&self, device_id: &str, button: Button, pressed: bool) -> Result<(), String> {
        let resolved_id = self.resolve_device_id(device_id)?;
        let mut devices = self.devices.lock();
        if let Some(device) = devices.get_mut(&resolved_id) {
            let flag = button.to_flag();
            if pressed {
                device.info.state.buttons |= flag;
            } else {
                device.info.state.buttons &= !flag;
            }
            match button {
                Button::LT => {
                    device.info.state.left_trigger = if pressed { 255 } else { 0 };
                }
                Button::RT => {
                    device.info.state.right_trigger = if pressed { 255 } else { 0 };
                }
                _ => {}
            }
            let device_snapshot = device.clone();
            let state = device.info.state.clone();
            let resolved = resolved_id.clone();
            drop(devices);

            // 如果正在录制宏，则记录事件
            if self.macro_recorder.is_recording() {
                let event = if pressed {
                    MacroEventType::ButtonPress(button)
                } else {
                    MacroEventType::ButtonRelease(button)
                };
                let _ = self.macro_recorder.add_event(event);
            }

            self.submit_report(&device_snapshot);
            self.emit_state_changed(&resolved, &state);
            Ok(())
        } else {
            Err(format!("设备不存在: {}", resolved_id))
        }
    }

    pub fn set_thumb(
        &self,
        device_id: &str,
        axis: ThumbAxis,
        value: f32,
    ) -> Result<(), String> {
        let resolved_id = self.resolve_device_id(device_id)?;
        let mut devices = self.devices.lock();
        if let Some(device) = devices.get_mut(&resolved_id) {
            let clamped = value.clamp(-1.0, 1.0);
            let scaled = (clamped * 32767.0) as i16;
            match axis {
                ThumbAxis::LeftX => device.info.state.left_thumb_x = scaled,
                ThumbAxis::LeftY => device.info.state.left_thumb_y = scaled,
                ThumbAxis::RightX => device.info.state.right_thumb_x = scaled,
                ThumbAxis::RightY => device.info.state.right_thumb_y = scaled,
            }
            let device_snapshot = device.clone();
            let state = device.info.state.clone();
            let resolved = resolved_id.clone();
            drop(devices);

            // 如果正在录制宏，则记录事件
            if self.macro_recorder.is_recording() {
                let (stick_name, x, y) = match axis {
                    ThumbAxis::LeftX | ThumbAxis::LeftY => (
                        "left".to_string(),
                        state.left_thumb_x as f32 / 32767.0,
                        state.left_thumb_y as f32 / 32767.0,
                    ),
                    ThumbAxis::RightX | ThumbAxis::RightY => (
                        "right".to_string(),
                        state.right_thumb_x as f32 / 32767.0,
                        state.right_thumb_y as f32 / 32767.0,
                    ),
                };
                let _ = self.macro_recorder.add_event(MacroEventType::ThumbMove(stick_name, x, y));
            }

            self.submit_report(&device_snapshot);
            self.emit_state_changed(&resolved, &state);
            Ok(())
        } else {
            Err(format!("设备不存在: {}", resolved_id))
        }
    }

    pub fn set_trigger(
        &self,
        device_id: &str,
        trigger: TriggerSide,
        value: f32,
    ) -> Result<(), String> {
        let resolved_id = self.resolve_device_id(device_id)?;
        let mut devices = self.devices.lock();
        if let Some(device) = devices.get_mut(&resolved_id) {
            let clamped = value.clamp(0.0, 1.0);
            let scaled = (clamped * 255.0) as u8;
            match trigger {
                TriggerSide::Left => device.info.state.left_trigger = scaled,
                TriggerSide::Right => device.info.state.right_trigger = scaled,
            }
            let device_snapshot = device.clone();
            let state = device.info.state.clone();
            let resolved = resolved_id.clone();
            drop(devices);

            // 如果正在录制宏，则记录事件
            if self.macro_recorder.is_recording() {
                let side_str = match trigger {
                    TriggerSide::Left => "left".to_string(),
                    TriggerSide::Right => "right".to_string(),
                };
                let _ = self.macro_recorder.add_event(MacroEventType::TriggerMove(side_str, clamped));
            }

            self.submit_report(&device_snapshot);
            self.emit_state_changed(&resolved, &state);
            Ok(())
        } else {
            Err(format!("设备不存在: {}", resolved_id))
        }
    }

    pub fn get_state(&self, device_id: &str) -> Result<ControllerState, String> {
        let resolved_id = self.resolve_device_id(device_id)?;
        let devices = self.devices.lock();
        devices
            .get(&resolved_id)
            .map(|d| d.info.state.clone())
            .ok_or_else(|| format!("设备不存在: {}", resolved_id))
    }

    pub fn list_devices(&self) -> Vec<DeviceInfo> {
        let devices = self.devices.lock();
        let mut list: Vec<DeviceInfo> = devices.values().map(|d| d.info.clone()).collect();
        list.sort_by_key(|d| d.index);
        list
    }

    pub fn reset_all_devices(&self) {
        let mut devices = self.devices.lock();
        let keys: Vec<String> = devices.keys().cloned().collect();
        for key in keys {
            if let Some(dev) = devices.get_mut(&key) {
                dev.info.state.buttons = 0;
                dev.info.state.left_trigger = 0;
                dev.info.state.right_trigger = 0;
                dev.info.state.left_thumb_x = 0;
                dev.info.state.left_thumb_y = 0;
                dev.info.state.right_thumb_x = 0;
                dev.info.state.right_thumb_y = 0;
                let dev_snapshot = dev.clone();
                drop(devices);
                self.submit_report(&dev_snapshot);
                self.emit_state_changed(&key, &dev_snapshot.info.state);
                devices = self.devices.lock();
            }
        }
    }

    pub fn get_vigem_status(&self) -> ViGEmStatus {
        let bindings = self.vigem_bindings.lock();
        let vigem = self.vigem.lock();
        let error_code = self.vigem_error_code.lock();

        let dll_found = bindings.is_some();
        let connected = vigem.is_some();
        let error_code_val = *error_code;

        let message = if connected {
            "ViGEmBus 已连接，虚拟手柄将被系统识别".to_string()
        } else if dll_found {
            match error_code_val {
                Some(1) => "ViGEmBus 驱动未找到（错误码: 1）。请安装 ViGEmBus 驱动后重启电脑".to_string(),
                Some(2) => "无法访问 ViGEmBus 驱动（错误码: 2）。请尝试以管理员身份运行程序".to_string(),
                Some(code) => format!("ViGEmBus 连接失败（错误码: {} / 0x{:08X}）。请确认驱动已正确安装", code, code),
                None => "ViGEmClient.dll 已找到，但驱动连接失败。请确认已安装 ViGEmBus 驱动".to_string(),
            }
        } else {
            "ViGEmClient.dll 未找到，使用模拟模式。请将 ViGEmClient.dll 放到程序目录下".to_string()
        };

        ViGEmStatus {
            available: connected,
            driver_installed: connected,
            dll_found,
            connected,
            error_code: error_code_val,
            message,
        }
    }

    pub fn try_connect_vigem(&self) -> Result<(), String> {
        let bindings = self.vigem_bindings.lock();
        if let Some(ref bindings_arc) = *bindings {
            let client = ViGEmClient::new(bindings_arc.clone())
                .map_err(|(code, msg)| {
                    let mut error_code = self.vigem_error_code.lock();
                    *error_code = Some(code);
                    msg
                })?;
            let mut vigem = self.vigem.lock();
            *vigem = Some(client);
            {
                let mut error_code = self.vigem_error_code.lock();
                *error_code = None;
            }
            tracing::info!("ViGEmBus 重新连接成功");
            Ok(())
        } else {
            Err("ViGEmClient.dll 未加载，无法连接".to_string())
        }
    }
}

impl Clone for ManagedDevice {
    fn clone(&self) -> Self {
        Self {
            info: self.info.clone(),
            target: self.target,
        }
    }
}
