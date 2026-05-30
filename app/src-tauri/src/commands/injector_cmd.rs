// app/src-tauri/src/commands/injector_cmd.rs
// Tauri 2.0 防止失去焦点功能后端命令路由

use crate::system::injector::{list_windowed_processes, run_injector_inject, run_injector_unload, ProcessInfo};

#[tauri::command]
pub fn get_injectable_processes() -> Result<Vec<ProcessInfo>, String> {
    Ok(list_windowed_processes())
}

#[tauri::command]
pub fn inject_focus_hook(pid: u32, is_64bit: bool) -> Result<(), String> {
    run_injector_inject(pid, is_64bit)
}

#[tauri::command]
pub fn unload_focus_hook(pid: u32, is_64bit: bool) -> Result<(), String> {
    run_injector_unload(pid, is_64bit)
}

#[tauri::command]
pub fn check_is_admin() -> bool {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        std::process::Command::new("net")
            .arg("session")
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }
    #[cfg(not(target_os = "windows"))]
    {
        true
    }
}

