use crate::script_engine::{Script, ScriptMeta, ScriptRuntime, ScriptValidationResult};

#[tauri::command]
pub fn script_create(
    runtime: tauri::State<'_, ScriptRuntime>,
    name: String,
    code: String,
) -> Script {
    runtime.create_script(&name, &code)
}

#[tauri::command]
pub fn script_execute(
    runtime: tauri::State<'_, ScriptRuntime>,
    script_id: String,
) -> Result<String, String> {
    runtime.execute_script(&script_id)
}

#[tauri::command]
pub fn script_validate_code(
    runtime: tauri::State<'_, ScriptRuntime>,
    code: String,
) -> ScriptValidationResult {
    runtime.validate_script_code(&code)
}

#[tauri::command]
pub fn script_debug_execute(
    runtime: tauri::State<'_, ScriptRuntime>,
    script_id: String,
    breakpoints: Vec<usize>,
) -> Result<String, String> {
    runtime.debug_script(&script_id, breakpoints)
}

#[tauri::command]
pub fn script_debug_resume(
    runtime: tauri::State<'_, ScriptRuntime>,
    execution_id: String,
) -> Result<(), String> {
    runtime.debug_resume(&execution_id)
}

#[tauri::command]
pub fn script_debug_step(
    runtime: tauri::State<'_, ScriptRuntime>,
    execution_id: String,
) -> Result<(), String> {
    runtime.debug_step(&execution_id)
}

#[tauri::command]
pub fn script_debug_stop(
    runtime: tauri::State<'_, ScriptRuntime>,
    execution_id: String,
) -> Result<(), String> {
    runtime.stop_execution(&execution_id)
}

#[tauri::command]
pub fn script_stop(
    runtime: tauri::State<'_, ScriptRuntime>,
    execution_id: String,
) -> Result<(), String> {
    runtime.stop_execution(&execution_id)
}

#[tauri::command]
pub fn script_list(runtime: tauri::State<'_, ScriptRuntime>) -> Vec<ScriptMeta> {
    runtime.list_scripts()
}

#[tauri::command]
pub fn script_get(runtime: tauri::State<'_, ScriptRuntime>, script_id: String) -> Option<Script> {
    runtime.get_script(&script_id)
}

#[tauri::command]
pub fn script_update(
    runtime: tauri::State<'_, ScriptRuntime>,
    script_id: String,
    code: String,
) -> Result<Script, String> {
    runtime.update_script(&script_id, &code)
}

#[tauri::command]
pub fn script_rename(
    runtime: tauri::State<'_, ScriptRuntime>,
    script_id: String,
    new_name: String,
) -> Result<Script, String> {
    runtime.rename_script(&script_id, &new_name)
}

#[tauri::command]
pub fn script_delete(
    runtime: tauri::State<'_, ScriptRuntime>,
    script_id: String,
) -> Result<(), String> {
    runtime.delete_script(&script_id)
}
