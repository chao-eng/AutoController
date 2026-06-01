// app/src-tauri/src/commands/injector_cmd.rs
// Tauri 2.0 防止失去焦点功能后端命令路由

use crate::system::injector::{
    list_windowed_processes, run_injector_inject, run_injector_unload, ProcessInfo, InjectedProcessesState
};

#[tauri::command]
pub fn get_injectable_processes(
    state: tauri::State<'_, InjectedProcessesState>
) -> Result<Vec<ProcessInfo>, String> {
    // 1. 获取当前所有活动窗口进程
    let mut list = list_windowed_processes();
    
    // 2. 检查已注入状态中的进程是否还在运行。如果进程退出了，我们把它从已注入列表中剔除
    let active_pids: std::collections::HashSet<u32> = list.iter().map(|p| p.pid).collect();
    {
        let mut injected = state.processes.lock();
        injected.retain(|pid, _| active_pids.contains(pid));
    }
    
    // 3. 排除已经被注入的进程，让它们不在左侧列表中显示
    let injected = state.processes.lock();
    list.retain(|p| !injected.contains_key(&p.pid));
    
    Ok(list)
}

#[tauri::command]
pub fn get_injected_processes(
    state: tauri::State<'_, InjectedProcessesState>
) -> Result<Vec<ProcessInfo>, String> {
    // 1. 获取当前所有活动窗口进程，用于确定哪些被注入进程依然存活
    let list = list_windowed_processes();
    let active_pids: std::collections::HashSet<u32> = list.iter().map(|p| p.pid).collect();
    
    // 2. 自动清理已经关闭退出的进程
    {
        let mut injected = state.processes.lock();
        injected.retain(|pid, _| active_pids.contains(pid));
    }
    
    // 3. 返回仍在运行的已附加 Hook 进程列表
    let injected = state.processes.lock();
    let mut result: Vec<ProcessInfo> = injected.values().cloned().collect();
    result.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(result)
}

#[tauri::command]
pub fn inject_focus_hook(
    state: tauri::State<'_, InjectedProcessesState>,
    pid: u32,
    is64bit: bool,
) -> Result<(), String> {
    // 运行物理隔离注入器
    run_injector_inject(pid, is64bit)?;
    
    // 注入成功后，在系统中找到对应的 ProcessInfo 详情并存入全局状态中，实现前端持久回显
    let list = list_windowed_processes();
    let proc_info = list.into_iter().find(|p| p.pid == pid).unwrap_or_else(|| {
        ProcessInfo {
            pid,
            name: "未知进程".to_string(),
            window_title: "未知窗口".to_string(),
            is_64bit: is64bit,
        }
    });
    
    let mut injected = state.processes.lock();
    injected.insert(pid, proc_info);
    
    Ok(())
}

#[tauri::command]
pub fn unload_focus_hook(
    state: tauri::State<'_, InjectedProcessesState>,
    pid: u32,
    is64bit: bool,
) -> Result<(), String> {
    // 运行物理隔离卸载器
    run_injector_unload(pid, is64bit)?;
    
    // 卸载成功后，从全局状态中移除
    let mut injected = state.processes.lock();
    injected.remove(&pid);
    
    Ok(())
}

#[tauri::command]
pub fn check_is_admin() -> bool {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        std::process::Command::new("net")
            .arg("session")
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }
    #[cfg(not(target_os = "windows"))]
    {
        true
    }
}

#[tauri::command]
pub fn add_defender_exclusion() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        let current_exe = std::env::current_exe().map_err(|e| format!("无法获取当前可执行文件路径: {}", e))?;
        let dir = current_exe.parent().ok_or("无法获取当前程序的运行目录")?;
        let path_str = dir.to_string_lossy();
        
        let mut cmd = std::process::Command::new("powershell");
        cmd.arg("-Command")
           .arg(format!("Add-MpPreference -ExclusionPath '{}'", path_str));
        
        // 隐藏命令行窗口
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        
        let output = cmd.output().map_err(|e| format!("启动 PowerShell 失败: {}", e))?;
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("PowerShell 执行失败: {}", stderr))
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err("此功能仅在 Windows 系统中可用".to_string())
    }
}


