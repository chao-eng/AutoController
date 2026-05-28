use crate::macro_engine::{MacroRecorder, MacroPlayer, Macro, MacroMeta};
use crate::controller::ControllerManager;
use crate::script_engine::ScriptRuntime;

fn button_to_str(btn: &crate::controller::Button) -> &'static str {
    match btn {
        crate::controller::Button::A => "A",
        crate::controller::Button::B => "B",
        crate::controller::Button::X => "X",
        crate::controller::Button::Y => "Y",
        crate::controller::Button::LB => "LB",
        crate::controller::Button::RB => "RB",
        crate::controller::Button::LT => "LT",
        crate::controller::Button::RT => "RT",
        crate::controller::Button::Back => "BACK",
        crate::controller::Button::Start => "START",
        crate::controller::Button::Guide => "GUIDE",
        crate::controller::Button::LeftThumb => "LS",
        crate::controller::Button::RightThumb => "RS",
        crate::controller::Button::DPadUp => "UP",
        crate::controller::Button::DPadDown => "DOWN",
        crate::controller::Button::DPadLeft => "LEFT",
        crate::controller::Button::DPadRight => "RIGHT",
    }
}

fn convert_macro_to_script(mac: &Macro) -> String {
    let mut code = String::new();
    code.push_str(&format!("// 自动由宏录制转换的脚本: {}\n", mac.name));
    code.push_str(&format!("// 录制日期: {}\n", mac.created_at.to_rfc3339()));
    code.push_str(&format!("// 事件总数: {}\n", mac.events.len()));
    code.push_str(&format!("// 时长: {} ms\n\n", mac.total_duration_ms));
    code.push_str("set_default_device(0);\n\n");

    let mut prev_time = 0;
    for event in &mac.events {
        let delta = event.timestamp_ms.saturating_sub(prev_time);
        if delta > 0 {
            code.push_str(&format!("sleep({});\n", delta));
        }
        prev_time = event.timestamp_ms;

        match &event.event_type {
            crate::macro_engine::types::MacroEventType::ButtonPress(btn) => {
                code.push_str(&format!("press(\"{}\");\n", button_to_str(btn)));
            }
            crate::macro_engine::types::MacroEventType::ButtonRelease(btn) => {
                code.push_str(&format!("release(\"{}\");\n", button_to_str(btn)));
            }
            crate::macro_engine::types::MacroEventType::ThumbMove(stick, x, y) => {
                let (axis_x, axis_y) = if stick.to_lowercase() == "left" {
                    ("LeftX", "LeftY")
                } else {
                    ("RightX", "RightY")
                };
                code.push_str(&format!("set_thumb(\"{}\", {:.4});\n", axis_x, x));
                code.push_str(&format!("set_thumb(\"{}\", {:.4});\n", axis_y, y));
            }
            crate::macro_engine::types::MacroEventType::TriggerMove(side, value) => {
                let side_str = if side.to_lowercase() == "left" {
                    "Left"
                } else {
                    "Right"
                };
                code.push_str(&format!("set_trigger(\"{}\", {:.4});\n", side_str, value));
            }
        }
    }
    code
}

#[tauri::command]
pub fn macro_start_record(
    recorder: tauri::State<'_, MacroRecorder>,
    device_id: String,
    name: String,
) -> Result<String, String> {
    recorder.start_record(&device_id, &name)
}

#[tauri::command]
pub fn macro_stop_record(
    recorder: tauri::State<'_, MacroRecorder>,
    runtime: tauri::State<'_, ScriptRuntime>,
) -> Result<Macro, String> {
    let mac = recorder.stop_record()?;

    // 自动转换为 Rhai 脚本并保存到脚本库
    let script_code = convert_macro_to_script(&mac);
    let script_name = format!("{} (自动转换)", mac.name);
    let _ = runtime.create_script(&script_name, &script_code);

    Ok(mac)
}

#[tauri::command]
pub fn macro_play(
    player: tauri::State<'_, MacroPlayer>,
    recorder: tauri::State<'_, MacroRecorder>,
    controller: tauri::State<'_, ControllerManager>,
    macro_id: String,
    speed: f32,
    loop_count: u32,
) -> Result<String, String> {
    let mac = recorder.get_macro(&macro_id).ok_or_else(|| "宏不存在".to_string())?;
    player.start_playback(controller.inner().clone(), mac, speed, loop_count)
}

#[tauri::command]
pub fn macro_pause(
    player: tauri::State<'_, MacroPlayer>,
    playback_id: String,
) -> Result<(), String> {
    player.pause(&playback_id)
}

#[tauri::command]
pub fn macro_resume(
    player: tauri::State<'_, MacroPlayer>,
    playback_id: String,
) -> Result<(), String> {
    player.resume(&playback_id)
}

#[tauri::command]
pub fn macro_stop(
    player: tauri::State<'_, MacroPlayer>,
    playback_id: String,
) -> Result<(), String> {
    player.stop(&playback_id)
}

#[tauri::command]
pub fn macro_list(
    recorder: tauri::State<'_, MacroRecorder>,
) -> Vec<MacroMeta> {
    recorder.list_macros()
}

#[tauri::command]
pub fn macro_delete(
    recorder: tauri::State<'_, MacroRecorder>,
    macro_id: String,
) -> Result<(), String> {
    recorder.delete_macro(&macro_id)
}

#[derive(serde::Serialize)]
pub struct XInputStatus {
    pub available: bool,
    pub error: Option<String>,
    pub connected_devices: Vec<u32>,
}

#[tauri::command]
pub fn macro_xinput_status() -> XInputStatus {
    use crate::macro_engine::xinput::XInputBindings;
    match XInputBindings::load() {
        Ok(bindings) => {
            let mut connected = Vec::new();
            for i in 0..4 {
                if bindings.get_state(i).is_ok() {
                    connected.push(i);
                }
            }
            XInputStatus {
                available: true,
                error: None,
                connected_devices: connected,
            }
        }
        Err(e) => {
            XInputStatus {
                available: false,
                error: Some(e),
                connected_devices: Vec::new(),
            }
        }
    }
}
