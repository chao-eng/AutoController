use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use super::types::*;
use super::xinput::{XInputBindings, XinputGamepad};
use crate::persistence::DataDir;
use crate::controller::types::Button;
use chrono::Utc;

#[derive(Clone)]
pub struct MacroRecorder {
    recording: Arc<Mutex<Option<RecordingState>>>,
    macros: Arc<Mutex<HashMap<String, Macro>>>,
    data_dir: Arc<DataDir>,
}

#[allow(dead_code)]
struct RecordingState {
    id: String,
    name: String,
    device_id: String,
    start_time: Instant,
    events: Vec<MacroEvent>,
}

impl MacroRecorder {
    pub fn new() -> Self {
        let data_dir = Arc::new(DataDir::new());
        let macros = match data_dir.load::<HashMap<String, Macro>>("macros") {
            Some(data) => Arc::new(Mutex::new(data)),
            None => Arc::new(Mutex::new(HashMap::new())),
        };
        Self {
            recording: Arc::new(Mutex::new(None)),
            macros,
            data_dir,
        }
    }

    fn persist(&self) {
        let macros = self.macros.lock();
        if let Err(e) = self.data_dir.save("macros", &*macros) {
            tracing::warn!(error = %e, "宏数据持久化失败");
        }
    }

    pub fn start_record(&self, device_id: &str, name: &str) -> Result<String, String> {
        let mut recording = self.recording.lock();
        if recording.is_some() {
            return Err("已有录制正在进行".to_string());
        }
        let id = uuid::Uuid::new_v4().to_string();
        *recording = Some(RecordingState {
            id: id.clone(),
            name: name.to_string(),
            device_id: device_id.to_string(),
            start_time: Instant::now(),
            events: Vec::new(),
        });

        // 启动后台物理 XInput 手柄捕获线程
        let recording_clone = self.recording.clone();
        let device_id_clone = device_id.to_string();

        std::thread::spawn(move || {
            tracing::info!("XInput 物理手柄后台捕获线程已启动");
            let bindings = match XInputBindings::load() {
                Ok(b) => b,
                Err(e) => {
                    tracing::warn!("无法加载 XInput 驱动，物理手柄捕获不可用: {}", e);
                    return;
                }
            };

            let mut prev_gamepads: [Option<XinputGamepad>; 4] = [None, None, None, None];
            let mut last_warn_time = std::time::Instant::now();
            let poll_interval = std::time::Duration::from_millis(16); // ~60Hz 采样率

            loop {
                // 检查录制是否仍在运行
                {
                    let rec = recording_clone.lock();
                    if rec.is_none() {
                        break;
                    }
                }

                let mut has_any_connected = false;

                for i in 0..4 {
                    if let Ok(state) = bindings.get_state(i as u32) {
                        has_any_connected = true;
                        let gamepad = state.gamepad;
                        if let Some(prev) = prev_gamepads[i] {
                            if gamepad != prev {
                                let timestamp_ms = {
                                    let rec = recording_clone.lock();
                                    if let Some(ref r_state) = *rec {
                                        r_state.start_time.elapsed().as_millis() as u64
                                    } else {
                                        break;
                                    }
                                };

                                let mut new_events = Vec::new();

                                // 1. 监测按键变化
                                let buttons_to_check = [
                                    (0x1000, Button::A),
                                    (0x2000, Button::B),
                                    (0x4000, Button::X),
                                    (0x8000, Button::Y),
                                    (0x0100, Button::LB),
                                    (0x0200, Button::RB),
                                    (0x0020, Button::Back),
                                    (0x0010, Button::Start),
                                    (0x0040, Button::LeftThumb),
                                    (0x0080, Button::RightThumb),
                                    (0x0001, Button::DPadUp),
                                    (0x0002, Button::DPadDown),
                                    (0x0004, Button::DPadLeft),
                                    (0x0008, Button::DPadRight),
                                ];

                                for &(flag, btn) in &buttons_to_check {
                                    let was_pressed = (prev.w_buttons & flag) != 0;
                                    let is_pressed = (gamepad.w_buttons & flag) != 0;
                                    if is_pressed != was_pressed {
                                        new_events.push(MacroEvent {
                                            timestamp_ms,
                                            device_id: device_id_clone.clone(),
                                            event_type: if is_pressed {
                                                MacroEventType::ButtonPress(btn)
                                            } else {
                                                MacroEventType::ButtonRelease(btn)
                                            },
                                        });
                                    }
                                }

                                // 2. 监测扳机键变化（防抖去噪滤波，变化量 > 2/255 才记录）
                                if (gamepad.b_left_trigger as i16 - prev.b_left_trigger as i16).abs() > 2 {
                                    new_events.push(MacroEvent {
                                        timestamp_ms,
                                        device_id: device_id_clone.clone(),
                                        event_type: MacroEventType::TriggerMove("left".to_string(), gamepad.b_left_trigger as f32 / 255.0),
                                    });
                                }
                                if (gamepad.b_right_trigger as i16 - prev.b_right_trigger as i16).abs() > 2 {
                                    new_events.push(MacroEvent {
                                        timestamp_ms,
                                        device_id: device_id_clone.clone(),
                                        event_type: MacroEventType::TriggerMove("right".to_string(), gamepad.b_right_trigger as f32 / 255.0),
                                    });
                                }

                                // 3. 监测摇杆变化（防抖去噪滤波，偏转增量 > 300 才记录）
                                if (gamepad.s_thumb_lx as i32 - prev.s_thumb_lx as i32).abs() > 300 ||
                                   (gamepad.s_thumb_ly as i32 - prev.s_thumb_ly as i32).abs() > 300 {
                                    new_events.push(MacroEvent {
                                        timestamp_ms,
                                        device_id: device_id_clone.clone(),
                                        event_type: MacroEventType::ThumbMove(
                                            "left".to_string(),
                                            gamepad.s_thumb_lx as f32 / 32767.0,
                                            gamepad.s_thumb_ly as f32 / 32767.0,
                                        ),
                                    });
                                }
                                if (gamepad.s_thumb_rx as i32 - prev.s_thumb_rx as i32).abs() > 300 ||
                                   (gamepad.s_thumb_ry as i32 - prev.s_thumb_ry as i32).abs() > 300 {
                                    new_events.push(MacroEvent {
                                        timestamp_ms,
                                        device_id: device_id_clone.clone(),
                                        event_type: MacroEventType::ThumbMove(
                                            "right".to_string(),
                                            gamepad.s_thumb_rx as f32 / 32767.0,
                                            gamepad.s_thumb_ry as f32 / 32767.0,
                                        ),
                                    });
                                }

                                // 实时追加事件到正在录制的宏数据中
                                if !new_events.is_empty() {
                                    let mut rec = recording_clone.lock();
                                    if let Some(ref mut r_state) = *rec {
                                        r_state.events.extend(new_events);
                                    }
                                }
                            }
                        }
                        prev_gamepads[i] = Some(gamepad);
                    } else {
                        // 设备断开时重置，以避免下次重新连接时产生误触发的巨大跳变
                        prev_gamepads[i] = None;
                    }
                }

                if !has_any_connected {
                    if last_warn_time.elapsed() > std::time::Duration::from_secs(3) {
                        tracing::warn!("未检测到任何可用的物理手柄连接，请确认手柄已打开并连接到电脑！");
                        last_warn_time = std::time::Instant::now();
                    }
                }

                std::thread::sleep(poll_interval);
            }
            tracing::info!("XInput 物理手柄后台捕获线程已正常退出");
        });

        tracing::info!(macro_id = %id, device_id = %device_id, "开始录制宏");
        Ok(id)
    }

    pub fn stop_record(&self) -> Result<Macro, String> {
        let mut recording = self.recording.lock();
        let state = recording
            .take()
            .ok_or("没有正在进行的录制")?;

        let total_duration_ms = state.start_time.elapsed().as_millis() as u64;
        let mac = Macro {
            id: state.id.clone(),
            name: state.name,
            created_at: Utc::now(),
            total_duration_ms,
            events: state.events,
        };

        let mut macros = self.macros.lock();
        macros.insert(state.id, mac.clone());
        drop(macros);
        self.persist();
        tracing::info!(macro_id = %mac.id, duration_ms = total_duration_ms, "宏录制完成");
        Ok(mac)
    }

    pub fn add_event(&self, event_type: MacroEventType) -> Result<(), String> {
        let mut recording = self.recording.lock();
        if let Some(state) = recording.as_mut() {
            let timestamp_ms = state.start_time.elapsed().as_millis() as u64;
            state.events.push(MacroEvent {
                timestamp_ms,
                device_id: state.device_id.clone(),
                event_type,
            });
            Ok(())
        } else {
            Err("没有正在进行的录制".to_string())
        }
    }

    pub fn is_recording(&self) -> bool {
        self.recording.lock().is_some()
    }

    pub fn list_macros(&self) -> Vec<MacroMeta> {
        let macros = self.macros.lock();
        macros
            .values()
            .map(|m| MacroMeta {
                id: m.id.clone(),
                name: m.name.clone(),
                created_at: m.created_at,
                total_duration_ms: m.total_duration_ms,
                event_count: m.events.len(),
            })
            .collect()
    }

    pub fn get_macro(&self, id: &str) -> Option<Macro> {
        let macros = self.macros.lock();
        macros.get(id).cloned()
    }

    pub fn delete_macro(&self, id: &str) -> Result<(), String> {
        let mut macros = self.macros.lock();
        if macros.remove(id).is_some() {
            drop(macros);
            self.persist();
            Ok(())
        } else {
            Err(format!("宏不存在: {}", id))
        }
    }
}
