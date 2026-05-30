// app/src-tauri/src/system/injector.rs
// 系统底层注入器调用，运行安全的进程扫描，并通过物理隔离的独立进程 injector.exe 执行注入和卸载操作。

use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::process::Command;
use serde::Serialize;

use windows::Win32::Foundation::{HWND, LPARAM, BOOL, CloseHandle};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, IsWindowVisible, GetWindowTextW, GetWindowThreadProcessId,
};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW,
    TH32CS_SNAPPROCESS, PROCESSENTRY32W,
};
use windows::Win32::System::Threading::{
    OpenProcess, IsWow64Process, PROCESS_QUERY_LIMITED_INFORMATION,
};

#[derive(Serialize, Clone, Debug)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub window_title: String,
    pub is_64bit: bool,
}

// 缓存正在运行的 top-level 窗口及其对应的 PID 映射
struct EnumWindowsData {
    windows: Vec<(u32, String)>,
}

unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let data = &mut *(lparam.0 as *mut EnumWindowsData);
    
    // 只考虑可见的窗口
    if !IsWindowVisible(hwnd).as_bool() {
        return BOOL(1);
    }

    // 获取窗口标题
    let mut title_buf = [0u16; 512];
    let len = GetWindowTextW(hwnd, &mut title_buf);
    if len == 0 {
        return BOOL(1);
    }
    
    let title = String::from_utf16_lossy(&title_buf[..len as usize]);
    
    // 忽略一些常见的空标题、系统浮动条
    if title.trim().is_empty() {
        return BOOL(1);
    }

    let mut pid = 0u32;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));
    
    if pid != 0 {
        data.windows.push((pid, title));
    }

    BOOL(1)
}

// 判断进程架构是否为 64位
fn is_process_64bit(pid: u32) -> bool {
    unsafe {
        if let Ok(h_process) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            let mut is_wow64 = BOOL::default();
            if IsWow64Process(h_process, &mut is_wow64).is_ok() {
                let _ = CloseHandle(h_process);
                return !is_wow64.as_bool(); // wow64=false 表示 64 位（在 64 位操作系统上）
            }
            let _ = CloseHandle(h_process);
        }
    }
    true // 默认视为 64位
}

/// 安全地获取可注入进程列表（无敏感 API，绝不报毒）
pub fn list_windowed_processes() -> Vec<ProcessInfo> {
    let mut process_map = HashMap::new();
    
    unsafe {
        // 1. 获取所有活动进程的 PID 和 进程名称 映射
        if let Ok(h_snap) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) {
            let mut pe = PROCESSENTRY32W::default();
            pe.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

            if Process32FirstW(h_snap, &mut pe).is_ok() {
                loop {
                    let len = pe.szExeFile.iter().position(|&c| c == 0).unwrap_or(pe.szExeFile.len());
                    let exe_name = String::from_utf16_lossy(&pe.szExeFile[..len]);
                    process_map.insert(pe.th32ProcessID, exe_name);
                    
                    if Process32NextW(h_snap, &mut pe).is_err() {
                        break;
                    }
                }
            }
            let _ = CloseHandle(h_snap);
        }

        // 2. 枚举所有拥有可见窗口的窗口标题与 PID
        let mut data = EnumWindowsData { windows: Vec::new() };
        let data_ptr = &mut data as *mut EnumWindowsData as isize;
        
        let _ = EnumWindows(Some(enum_windows_callback), LPARAM(data_ptr));

        // 3. 将结果整合成 ProcessInfo 列表
        let mut list = Vec::new();
        let mut seen_pids = HashMap::new(); // 避免同一个进程多个窗口重复显示

        for (pid, title) in data.windows {
            // 忽略当前 AutoController 自身进程，防止套娃注入
            if pid == std::process::id() {
                continue;
            }

            if seen_pids.contains_key(&pid) {
                continue;
            }

            if let Some(name) = process_map.get(&pid) {
                // 忽略常见的系统服务进程
                let name_lower = name.to_lowercase();
                if name_lower == "explorer.exe"
                    || name_lower == "svchost.exe"
                    || name_lower == "taskmgr.exe"
                    || name_lower == "tauri.exe"
                    || name_lower == "conhost.exe"
                    || name_lower == "cmd.exe"
                    || name_lower == "powershell.exe"
                {
                    continue;
                }

                seen_pids.insert(pid, true);

                let is_64 = is_process_64bit(pid);

                list.push(ProcessInfo {
                    pid,
                    name: name.clone(),
                    window_title: title,
                    is_64bit: is_64,
                });
            }
        }

        list.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        list
    }
}

/// 定位独立的 injector.exe 子进程物理路径
fn get_injector_exe_path() -> Result<PathBuf, String> {
    let current_exe = env::current_exe().map_err(|e| format!("无法获取当前程序运行路径: {}", e))?;
    let dir = current_exe.parent().ok_or("无法获取当前程序的运行目录")?;
    
    // 自适应开发环境（target/debug/）和打包发布安装环境
    let injector_path = dir.join("injector.exe");
    if injector_path.exists() {
        return Ok(injector_path);
    }

    // 备用开发环境寻找方式
    let alt_path = dir.join("deps").join("injector.exe");
    if alt_path.exists() {
        return Ok(alt_path);
    }

    Err(format!(
        "核心安全组件（injector.exe）缺失，已被杀毒软件隔离删除或尚未编译。预计路径: {:?}",
        injector_path
    ))
}

/// 唤起独立子进程执行注入（物理隔离脏活，规避主程序报毒崩溃）
pub fn run_injector_inject(pid: u32, is_64bit: bool) -> Result<(), String> {
    let injector_exe = get_injector_exe_path()?;
    
    // 定位 DLL 路径
    let current_exe = env::current_exe().map_err(|e| format!("无法获取当前程序路径: {}", e))?;
    let dir = current_exe.parent().ok_or("无法获取当前程序的运行目录")?;
    
    let dll_name = if is_64bit { "NoFocusLoss64.dll" } else { "NoFocusLoss.dll" };
    let dll_path = dir.join(dll_name);
    
    if !dll_path.exists() {
        return Err(format!(
            "防失去焦点核心 DLL 组件不存在，请确保 {} 放在软件目录下！",
            dll_name
        ));
    }

    // 启动独立进程
    let mut cmd = Command::new(&injector_exe);
    cmd.arg("--inject")
       .arg(pid.to_string())
       .arg(dll_path.to_string_lossy().to_string());

    // Windows 平台强力隐藏子进程的小黑框命令行窗口
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    let output = cmd.output().map_err(|e| {
        match e.kind() {
            std::io::ErrorKind::PermissionDenied => {
                "安全系统拦截：启动核心注入组件被杀毒软件（Windows Defender 等）阻止，请前往杀软排除拦截列表，将 injector.exe 添加为信任信任项！".to_string()
            }
            std::io::ErrorKind::NotFound => {
                "安全系统拦截：核心注入组件被杀毒软件检测并直接彻底隔离删除，请前往杀软恢复并添加信任项！".to_string()
            }
            _ => format!("启动注入组件子进程失败: {}", e),
        }
    })?;

    if output.status.success() {
        Ok(())
    } else {
        let code = output.status.code().unwrap_or(-1);
        let err_desc = match code {
            102 => "打开目标进程失败，可能权限不足，请右键选择「以管理员身份运行」本程序！",
            103 => "打开目标进程被拦截，可能已被防作弊系统监控！",
            104 => "在目标进程中开辟虚拟内存被杀软拦截！",
            105 => "向目标进程写入配置数据被拦截！",
            108 => "在目标进程中创建远程激活线程被杀毒软件（如 Defender）强力阻断拦截！",
            109 => "核心 DLL 模块在目标进程中加载初始化失败（请确保没有多开或目标进程未拒绝加载外部动态库）！",
            110 => "动态加载核心 Win32 API 被杀毒软件拦截阻断！",
            111 | 112 => "核心防失去焦点 DLL 未能正确被定位或路径包含非法字符！",
            _ => "未知注入错误，可能受到系统安全拦截，请以管理员身份重试！",
        };
        Err(format!("注入组件返回错误代码 ({}): {}", code, err_desc))
    }
}

/// 唤起独立子进程执行安全卸载
pub fn run_injector_unload(pid: u32, is_64bit: bool) -> Result<(), String> {
    let injector_exe = get_injector_exe_path()?;
    let dll_name = if is_64bit { "NoFocusLoss64.dll" } else { "NoFocusLoss.dll" };

    let mut cmd = Command::new(&injector_exe);
    cmd.arg("--unload")
       .arg(pid.to_string())
       .arg(dll_name);

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }

    let output = cmd.output().map_err(|e| {
        match e.kind() {
            std::io::ErrorKind::PermissionDenied => {
                "启动卸载核心组件被拦截，请在杀毒软件中予以信任项！".to_string()
            }
            _ => format!("启动卸载组件子进程失败: {}", e),
        }
    })?;

    if output.status.success() {
        Ok(())
    } else {
        let code = output.status.code().unwrap_or(-1);
        let err_desc = match code {
            102 | 103 => "打开目标进程失败，请确认该进程是否已被关闭！",
            121 => "在目标进程中未找到已附加的 NoFocusLoss 模块！",
            123 => "在目标进程中创建卸载线程被安全拦截！",
            _ => "未知卸载错误，请尝试以管理员身份运行本程序！",
        };
        Err(format!("卸载组件返回错误代码 ({}): {}", code, err_desc))
    }
}
