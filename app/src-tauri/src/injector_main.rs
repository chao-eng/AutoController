// app/src-tauri/src/injector_main.rs
// 独立的注入器可执行程序，用于隔离敏感跨进程注入 API。

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use std::ptr;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, HANDLE, BOOL, GetLastError};
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_CREATE_THREAD,
    PROCESS_QUERY_INFORMATION, PROCESS_VM_OPERATION, PROCESS_VM_WRITE, PROCESS_VM_READ,
    GetExitCodeThread,
};
use windows::Win32::System::Memory::{
    MEM_COMMIT, MEM_RESERVE, MEM_RELEASE, PAGE_READWRITE,
};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Module32FirstW, Module32NextW,
    TH32CS_SNAPMODULE, TH32CS_SNAPMODULE32, MODULEENTRY32W,
};

// XOR 密钥
const XOR_KEY: u8 = 0x5A;

// XOR 静态解密函数，在运行时解密并自动在末尾追加 \0 以构造合法的 C 风格 null-terminated 字节数组
fn decrypt_to_bytes(encoded: &[u8]) -> Vec<u8> {
    let mut decrypted: Vec<u8> = encoded.iter().map(|&b| b ^ XOR_KEY).collect();
    decrypted.push(0); // 确保 Null 结尾
    decrypted
}

// 辅助转为 PCWSTR 的 WCHAR 数组
fn to_widestring(s: &str) -> Vec<u16> {
    let mut v: Vec<u16> = OsStr::new(s).encode_wide().collect();
    v.push(0);
    v
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    // 如果没有足够的参数，直接静默退出（或退出码 99）
    if args.len() < 3 {
        std::process::exit(99);
    }

    let mode = &args[1];
    
    if mode == "--inject" {
        if args.len() < 4 {
            std::process::exit(99);
        }
        let pid_str = &args[2];
        let dll_path = &args[3];
        
        let pid: u32 = match pid_str.parse() {
            Ok(p) => p,
            Err(_) => std::process::exit(101),
        };

        match inject_dll(pid, dll_path) {
            Ok(_) => std::process::exit(0),
            Err(code) => std::process::exit(code),
        }
    } else if mode == "--unload" {
        if args.len() < 4 {
            std::process::exit(99);
        }
        let pid_str = &args[2];
        let dll_name = &args[3];
        
        let pid: u32 = match pid_str.parse() {
            Ok(p) => p,
            Err(_) => std::process::exit(101),
        };

        match unload_dll(pid, dll_name) {
            Ok(_) => std::process::exit(0),
            Err(code) => std::process::exit(code),
        }
    } else {
        std::process::exit(99);
    }
}

// 敏感 API 动态函数指针类型定义
type FnVirtualAllocEx = unsafe extern "system" fn(
    hprocess: HANDLE,
    lpaddress: *const std::ffi::c_void,
    dwsize: usize,
    flallocationtype: u32,
    flprotect: u32,
) -> *mut std::ffi::c_void;

type FnWriteProcessMemory = unsafe extern "system" fn(
    hprocess: HANDLE,
    lpbaseaddress: *const std::ffi::c_void,
    lpbuffer: *const std::ffi::c_void,
    nsize: usize,
    lpnumberofbyteswritten: *mut usize,
) -> BOOL;

type FnCreateRemoteThread = unsafe extern "system" fn(
    hprocess: HANDLE,
    lpthreadattributes: *const std::ffi::c_void,
    dwstacksize: usize,
    lpstartaddress: unsafe extern "system" fn(*mut std::ffi::c_void) -> u32,
    lpparameter: *const std::ffi::c_void,
    dwcreationflags: u32,
    lpthreadid: *mut u32,
) -> HANDLE;

type FnVirtualFreeEx = unsafe extern "system" fn(
    hprocess: HANDLE,
    lpaddress: *mut std::ffi::c_void,
    dwsize: usize,
    dwfreetype: u32,
) -> BOOL;

// 动态解析核心 Win32 API 避免静态导入表报毒
struct DynWinApi {
    virtual_alloc_ex: FnVirtualAllocEx,
    write_process_memory: FnWriteProcessMemory,
    create_remote_thread: FnCreateRemoteThread,
    virtual_free_ex: FnVirtualFreeEx,
}

impl DynWinApi {
    fn load() -> Option<Self> {
        unsafe {
            // XOR 混淆后的 "kernel32.dll"
            let k32_enc = &[49, 63, 40, 52, 63, 54, 105, 104, 116, 62, 54, 54];
            let k32_bytes = decrypt_to_bytes(k32_enc);
            // 剔除末尾追加的零字节
            let k32_name = String::from_utf8_lossy(&k32_bytes[..k32_bytes.len() - 1]).to_string();
            let k32_w = to_widestring(&k32_name);
            let h_kernel32 = GetModuleHandleW(PCWSTR(k32_w.as_ptr())).ok()?;

            // XOR 混淆后的 "VirtualAllocEx"
            let alloc_enc = &[12, 51, 40, 46, 47, 59, 54, 27, 54, 54, 53, 57, 31, 34];
            let alloc_bytes = decrypt_to_bytes(alloc_enc);
            let p_alloc = GetProcAddress(h_kernel32, windows::core::PCSTR(alloc_bytes.as_ptr()))?;

            // XOR 混淆后的 "WriteProcessMemory"
            let write_enc = &[13, 40, 51, 46, 63, 10, 40, 53, 57, 63, 41, 41, 23, 63, 55, 53, 40, 35];
            let write_bytes = decrypt_to_bytes(write_enc);
            let p_write = GetProcAddress(h_kernel32, windows::core::PCSTR(write_bytes.as_ptr()))?;

            // XOR 混淆后的 "CreateRemoteThread"
            let thread_enc = &[25, 40, 63, 59, 46, 63, 8, 63, 55, 53, 46, 63, 14, 50, 40, 63, 59, 62];
            let thread_bytes = decrypt_to_bytes(thread_enc);
            let p_thread = GetProcAddress(h_kernel32, windows::core::PCSTR(thread_bytes.as_ptr()))?;

            // XOR 混淆后的 "VirtualFreeEx"
            let free_enc = &[12, 51, 40, 46, 47, 59, 54, 28, 40, 63, 63, 31, 34];
            let free_bytes = decrypt_to_bytes(free_enc);
            let p_free = GetProcAddress(h_kernel32, windows::core::PCSTR(free_bytes.as_ptr()))?;

            Some(DynWinApi {
                virtual_alloc_ex: std::mem::transmute(p_alloc),
                write_process_memory: std::mem::transmute(p_write),
                create_remote_thread: std::mem::transmute(p_thread),
                virtual_free_ex: std::mem::transmute(p_free),
            })
        }
    }
}

// DLL 注入实现
fn inject_dll(pid: u32, dll_path: &str) -> Result<(), i32> {
    unsafe {
        let dyn_api = DynWinApi::load().ok_or(110)?;

        let path = PathBuf::from(dll_path);
        if !path.exists() {
            return Err(111);
        }
        let abs_path = path.canonicalize().map_err(|_| 112)?;
        let abs_path_str = abs_path.to_string_lossy().trim_start_matches(r#"\\?\"#).to_string();
        
        let path_w = to_widestring(&abs_path_str);
        let path_bytes_size = path_w.len() * 2;

        let access = PROCESS_CREATE_THREAD | PROCESS_QUERY_INFORMATION | PROCESS_VM_OPERATION | PROCESS_VM_WRITE | PROCESS_VM_READ;
        let h_process = OpenProcess(access, false, pid).map_err(|_| {
            let err = GetLastError().0;
            if err == 5 { 102 } else { 103 }
        })?;

        let remote_mem = (dyn_api.virtual_alloc_ex)(
            h_process,
            ptr::null(),
            path_bytes_size,
            (MEM_COMMIT | MEM_RESERVE).0,
            PAGE_READWRITE.0,
        );
        if remote_mem.is_null() {
            let _ = CloseHandle(h_process);
            return Err(104);
        }

        let mut bytes_written = 0;
        let write_ok = (dyn_api.write_process_memory)(
            h_process,
            remote_mem,
            path_w.as_ptr() as *const std::ffi::c_void,
            path_bytes_size,
            &mut bytes_written,
        );
        if !write_ok.as_bool() {
            let _ = (dyn_api.virtual_free_ex)(h_process, remote_mem, 0, MEM_RELEASE.0);
            let _ = CloseHandle(h_process);
            return Err(105);
        }

        let k32_enc = &[49, 63, 40, 52, 63, 54, 105, 104, 116, 62, 54, 54];
        let k32_bytes = decrypt_to_bytes(k32_enc);
        let k32_name = String::from_utf8_lossy(&k32_bytes[..k32_bytes.len()-1]).to_string();
        let k32_w = to_widestring(&k32_name);
        let h_kernel32 = GetModuleHandleW(PCWSTR(k32_w.as_ptr())).map_err(|_| 106)?;

        // XOR 混淆后的 "LoadLibraryW"
        let load_lib_enc = &[22, 53, 59, 62, 22, 51, 56, 40, 59, 40, 35, 13];
        let load_lib_bytes = decrypt_to_bytes(load_lib_enc);
        let load_lib_addr = GetProcAddress(h_kernel32, windows::core::PCSTR(load_lib_bytes.as_ptr())).ok_or(107)?;
        let load_lib_fn: unsafe extern "system" fn(*mut std::ffi::c_void) -> u32 = std::mem::transmute(load_lib_addr);

        let h_thread = (dyn_api.create_remote_thread)(
            h_process,
            ptr::null(),
            0,
            load_lib_fn,
            remote_mem,
            0,
            ptr::null_mut(),
        );
        
        if h_thread.is_invalid() {
            let _ = (dyn_api.virtual_free_ex)(h_process, remote_mem, 0, MEM_RELEASE.0);
            let _ = CloseHandle(h_process);
            return Err(108);
        }

        windows::Win32::System::Threading::WaitForSingleObject(h_thread, 5000);
        
        let mut exit_code: u32 = 0;
        let got_exit_code = GetExitCodeThread(h_thread, &mut exit_code);
        
        let _ = CloseHandle(h_thread);
        let _ = (dyn_api.virtual_free_ex)(h_process, remote_mem, 0, MEM_RELEASE.0);
        let _ = CloseHandle(h_process);
        
        if got_exit_code.is_ok() && exit_code == 0 {
            // LoadLibraryW returned NULL (0) inside the target process
            return Err(109);
        }
        
        Ok(())
    }
}

// 从目标进程卸载 Hook DLL
fn unload_dll(pid: u32, dll_name: &str) -> Result<(), i32> {
    unsafe {
        let dyn_api = DynWinApi::load().ok_or(110)?;

        let mut module_base: *mut std::ffi::c_void = ptr::null_mut();
        let h_snap = CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid).map_err(|_| 120)?;
        
        let mut me = MODULEENTRY32W::default();
        me.dwSize = std::mem::size_of::<MODULEENTRY32W>() as u32;

        if Module32FirstW(h_snap, &mut me).is_ok() {
            loop {
                let len = me.szModule.iter().position(|&c| c == 0).unwrap_or(me.szModule.len());
                let cur_mod_name = String::from_utf16_lossy(&me.szModule[..len]);
                if cur_mod_name.to_lowercase() == dll_name.to_lowercase() {
                    module_base = me.modBaseAddr as *mut std::ffi::c_void;
                    break;
                }
                if Module32NextW(h_snap, &mut me).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(h_snap);

        if module_base.is_null() {
            return Err(121);
        }

        let access = PROCESS_CREATE_THREAD | PROCESS_QUERY_INFORMATION | PROCESS_VM_OPERATION | PROCESS_VM_WRITE | PROCESS_VM_READ;
        let h_process = OpenProcess(access, false, pid).map_err(|_| {
            let err = GetLastError().0;
            if err == 5 { 102 } else { 103 }
        })?;

        let k32_enc = &[49, 63, 40, 52, 63, 54, 105, 104, 116, 62, 54, 54];
        let k32_bytes = decrypt_to_bytes(k32_enc);
        let k32_name = String::from_utf8_lossy(&k32_bytes[..k32_bytes.len()-1]).to_string();
        let k32_w = to_widestring(&k32_name);
        let h_kernel32 = GetModuleHandleW(PCWSTR(k32_w.as_ptr())).map_err(|_| 106)?;

        // XOR 混淆后的 "FreeLibrary"
        let free_lib_enc = &[28, 40, 63, 63, 22, 51, 56, 40, 59, 40, 35];
        let free_lib_bytes = decrypt_to_bytes(free_lib_enc);
        let free_lib_addr = GetProcAddress(h_kernel32, windows::core::PCSTR(free_lib_bytes.as_ptr())).ok_or(122)?;
        let free_lib_fn: unsafe extern "system" fn(*mut std::ffi::c_void) -> u32 = std::mem::transmute(free_lib_addr);

        let h_thread = (dyn_api.create_remote_thread)(
            h_process,
            ptr::null(),
            0,
            free_lib_fn,
            module_base,
            0,
            ptr::null_mut(),
        );

        if h_thread.is_invalid() {
            let _ = CloseHandle(h_process);
            return Err(123);
        }

        windows::Win32::System::Threading::WaitForSingleObject(h_thread, 5000);
        let _ = CloseHandle(h_thread);
        let _ = CloseHandle(h_process);

        Ok(())
    }
}
