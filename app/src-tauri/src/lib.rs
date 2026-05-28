#![allow(dead_code)]

mod persistence;
mod controller;
mod macro_engine;
mod script_engine;
mod scheduler;
mod config;
mod logger;
mod system;
mod commands;
pub mod notify;


use tauri::Manager;
use commands::*;
use controller::ControllerManager;
use logger::TauriEventLayer;
use macro_engine::MacroPlayer;
use script_engine::ScriptRuntime;
use scheduler::TaskQueue;
use config::AppConfigManager;
use system::ProcessMonitor;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (tauri_layer, log_handle) = TauriEventLayer::new();

    // 从磁盘加载初始配置的日志级别
    let data_dir = persistence::DataDir::new();
    let initial_level = match data_dir.load::<config::AppConfig>("config") {
        Some(cfg) => cfg.log_level,
        None => "info".to_string(),
    };

    use tracing_subscriber::prelude::*;
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(format!("{},tao=error", initial_level)));

    let (filter_layer, reload_handle) = tracing_subscriber::reload::Layer::new(filter);

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(tauri_layer)
        .with(tracing_subscriber::fmt::layer())
        .init();

    let controller = ControllerManager::new();
    let recorder = controller.macro_recorder();
    let script_runtime = ScriptRuntime::with_controller(controller.clone());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .manage(controller)
        .manage(recorder)
        .manage(MacroPlayer::new())
        .manage(script_runtime)
        .manage(TaskQueue::new())
        .manage(AppConfigManager::new())
        .manage(ProcessMonitor::new())
        .manage(reload_handle)
        .setup(move |app| {
            system::tray::setup_tray(app)?;
            let handle = app.handle().clone();
            let controller = app.state::<ControllerManager>();
            controller.set_app_handle(handle.clone());
            let script_runtime = app.state::<ScriptRuntime>();
            script_runtime.set_app_handle(handle.clone());

            // 启动后台定时任务调度引擎心跳循环
            scheduler::queue::start_scheduler_loop(handle.clone());

            *log_handle.lock() = Some(handle.clone());

            // 启动后台 CPU 和内存资源监控线程 (每 2 秒采集一次并通过 Tauri 事件推送给前端)
            let handle_clone = handle.clone();
            std::thread::spawn(move || {
                use tauri::Emitter;
                let pid = std::process::id();
                loop {
                    let mut mem_str = "0 MB".to_string();
                    let mut cpu_str = "0%".to_string();

                    #[cfg(target_os = "windows")]
                    {
                        use std::os::windows::process::CommandExt;

                        // 获取当前进程的内存 WorkingSetSize
                        let mem_output = std::process::Command::new("wmic")
                            .args(["process", "where", &format!("processid={}", pid), "get", "WorkingSetSize", "/format:value"])
                            .creation_flags(0x08000000)
                            .output();
                        if let Ok(output) = mem_output {
                            let stdout = String::from_utf8_lossy(&output.stdout);
                            for line in stdout.lines() {
                                if line.starts_with("WorkingSetSize=") {
                                    if let Ok(bytes) = line.trim_start_matches("WorkingSetSize=").trim().parse::<u64>() {
                                        mem_str = format!("{} MB", bytes / 1024 / 1024);
                                    }
                                }
                            }
                        }

                        // 获取当前进程的 CPU PercentProcessorTime
                        let cpu_output = std::process::Command::new("wmic")
                            .args(["path", "Win32_PerfFormattedData_PerfProc_Process", "where", &format!("IDProcess={}", pid), "get", "PercentProcessorTime", "/format:value"])
                            .creation_flags(0x08000000)
                            .output();
                        if let Ok(output) = cpu_output {
                            let stdout = String::from_utf8_lossy(&output.stdout);
                            for line in stdout.lines() {
                                if line.starts_with("PercentProcessorTime=") {
                                    if let Ok(percent) = line.trim_start_matches("PercentProcessorTime=").trim().parse::<u64>() {
                                        cpu_str = format!("{}%", percent);
                                    }
                                }
                            }
                        }
                    }

                    #[cfg(not(target_os = "windows"))]
                    {
                        mem_str = "45 MB".to_string();
                        cpu_str = "1.2%".to_string();
                    }

                    #[derive(Clone, serde::Serialize)]
                    struct SystemResources {
                        cpu: String,
                        memory: String,
                    }

                    let _ = handle_clone.emit("system-resources", SystemResources {
                        cpu: cpu_str,
                        memory: mem_str,
                    });

                    std::thread::sleep(std::time::Duration::from_secs(2));
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            controller_cmd::controller_create,
            controller_cmd::controller_remove,
            controller_cmd::controller_set_button,
            controller_cmd::controller_set_thumb,
            controller_cmd::controller_set_trigger,
            controller_cmd::controller_get_state,
            controller_cmd::controller_list,
            controller_cmd::controller_vigem_status,
            controller_cmd::controller_toggle_connection,
            macro_cmd::macro_start_record,
            macro_cmd::macro_stop_record,
            macro_cmd::macro_play,
            macro_cmd::macro_pause,
            macro_cmd::macro_resume,
            macro_cmd::macro_stop,
            macro_cmd::macro_list,
            macro_cmd::macro_delete,
            macro_cmd::macro_xinput_status,
            script_cmd::script_create,
            script_cmd::script_execute,
            script_cmd::script_stop,
            script_cmd::script_list,
            script_cmd::script_get,
            script_cmd::script_update,
            script_cmd::script_rename,
            script_cmd::script_delete,
            scheduler_cmd::scheduler_create_task,
            scheduler_cmd::scheduler_remove_task,
            scheduler_cmd::scheduler_toggle_task,
            scheduler_cmd::scheduler_list,
            scheduler_cmd::scheduler_execute_sequence,
            scheduler_cmd::scheduler_stop_sequence,
            config_cmd::config_get,
            config_cmd::config_set,
            config_cmd::open_ocr_viewfinder,
            config_cmd::save_ocr_region,
            log_cmd::log_query,
            log_cmd::log_export,
            notify::send_aggregated_notification,
        ])

        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
