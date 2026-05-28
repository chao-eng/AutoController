use crate::scheduler::{TaskQueue, ScheduledTask};
use crate::script_engine::ScriptRuntime;
use crate::scheduler::types::TaskAction;

#[tauri::command]
pub fn scheduler_create_task(
    queue: tauri::State<'_, TaskQueue>,
    task: ScheduledTask,
) -> Result<(), String> {
    queue.add_task(task)
}

#[tauri::command]
pub fn scheduler_remove_task(
    queue: tauri::State<'_, TaskQueue>,
    task_id: String,
) -> Result<(), String> {
    queue.remove_task(&task_id)
}

#[tauri::command]
pub fn scheduler_toggle_task(
    queue: tauri::State<'_, TaskQueue>,
    task_id: String,
    enabled: bool,
) -> Result<(), String> {
    queue.toggle_task(&task_id, enabled)
}

#[tauri::command]
pub fn scheduler_list(
    queue: tauri::State<'_, TaskQueue>,
) -> Vec<ScheduledTask> {
    queue.list_tasks()
}

#[tauri::command]
pub fn scheduler_execute_sequence(
    runtime: tauri::State<'_, ScriptRuntime>,
    queue: tauri::State<'_, TaskQueue>,
    task_id: String,
) -> Result<(), String> {
    let task = queue.get_task(&task_id).ok_or_else(|| "任务不存在".to_string())?;
    match task.action {
        TaskAction::ExecuteSequence { steps, task_loop_count } => {
            runtime.execute_sequence(&task_id, steps, task_loop_count)
        }
        _ => Err("该任务不是多脚本串联序列任务".to_string()),
    }
}

#[tauri::command]
pub fn scheduler_stop_sequence(
    runtime: tauri::State<'_, ScriptRuntime>,
    task_id: String,
) -> Result<(), String> {
    runtime.stop_sequence(&task_id)
}
