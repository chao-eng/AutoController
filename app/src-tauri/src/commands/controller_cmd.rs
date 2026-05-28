use crate::controller::{ControllerManager, ControllerType, Button, ThumbAxis, TriggerSide, DeviceInfo, ControllerState, ViGEmStatus};

#[tauri::command]
pub fn controller_create(
    manager: tauri::State<'_, ControllerManager>,
    controller_type: ControllerType,
) -> Result<DeviceInfo, String> {
    manager.create_device(controller_type)
}

#[tauri::command]
pub fn controller_remove(
    manager: tauri::State<'_, ControllerManager>,
    device_id: String,
) -> Result<(), String> {
    manager.remove_device(&device_id)
}

#[tauri::command]
pub fn controller_set_button(
    manager: tauri::State<'_, ControllerManager>,
    device_id: String,
    button: Button,
    pressed: bool,
) -> Result<(), String> {
    manager.set_button(&device_id, button, pressed)
}

#[tauri::command]
pub fn controller_set_thumb(
    manager: tauri::State<'_, ControllerManager>,
    device_id: String,
    axis: ThumbAxis,
    value: f32,
) -> Result<(), String> {
    manager.set_thumb(&device_id, axis, value)
}

#[tauri::command]
pub fn controller_set_trigger(
    manager: tauri::State<'_, ControllerManager>,
    device_id: String,
    trigger: TriggerSide,
    value: f32,
) -> Result<(), String> {
    manager.set_trigger(&device_id, trigger, value)
}

#[tauri::command]
pub fn controller_get_state(
    manager: tauri::State<'_, ControllerManager>,
    device_id: String,
) -> Result<ControllerState, String> {
    manager.get_state(&device_id)
}

#[tauri::command]
pub fn controller_list(
    manager: tauri::State<'_, ControllerManager>,
) -> Vec<DeviceInfo> {
    manager.list_devices()
}

#[tauri::command]
pub fn controller_vigem_status(
    manager: tauri::State<'_, ControllerManager>,
) -> ViGEmStatus {
    manager.get_vigem_status()
}

#[tauri::command]
pub fn controller_toggle_connection(
    manager: tauri::State<'_, ControllerManager>,
    device_id: String,
) -> Result<DeviceInfo, String> {
    manager.toggle_device_connection(&device_id)
}
