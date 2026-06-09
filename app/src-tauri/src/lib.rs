#![allow(dead_code)]

mod commands;
mod config;
mod controller;
mod fh6_telemetry;
mod logger;
mod macro_engine;
pub mod notify;
mod persistence;
mod scheduler;
mod script_engine;
mod system;

use commands::*;
use config::AppConfigManager;
use controller::ControllerManager;
use logger::TauriEventLayer;
use macro_engine::MacroPlayer;
use scheduler::TaskQueue;
use script_engine::ScriptRuntime;
use system::injector::InjectedProcessesState;
use system::ProcessMonitor;
use tauri::Manager;

fn open_telemetry_db() -> rusqlite::Connection {
    match fh6_telemetry::db::open() {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!(error = %e, "Forza telemetry database failed to open; using in-memory fallback");
            let conn = rusqlite::Connection::open_in_memory()
                .expect("failed to open in-memory telemetry database");
            if let Err(init_error) = fh6_telemetry::db::init(&conn) {
                tracing::error!(error = %init_error, "Forza telemetry in-memory database init failed");
            }
            conn
        }
    }
}

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
    let filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        tracing_subscriber::EnvFilter::new(format!("{},tao=error", initial_level))
    });

    let (filter_layer, reload_handle) = tracing_subscriber::reload::Layer::new(filter);

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(tauri_layer)
        .with(tracing_subscriber::fmt::layer())
        .init();

    let controller = ControllerManager::new();
    let recorder = controller.macro_recorder();
    let script_runtime = ScriptRuntime::with_controller(controller.clone());

    let loaded_settings = fh6_telemetry::settings::load();
    let auto_record = loaded_settings.auto_record;
    let telemetry_state: fh6_telemetry::Shared = std::sync::Arc::new(fh6_telemetry::AppState {
        db: std::sync::Mutex::new(open_telemetry_db()),
        session_manager: std::sync::Mutex::new(fh6_telemetry::session::SessionManager::new(
            auto_record,
        )),
        settings: std::sync::Mutex::new(loaded_settings),
    });

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
        .manage(InjectedProcessesState::new())
        .manage(reload_handle)
        .manage(telemetry_state.clone())
        .setup(move |app| {
            system::tray::setup_tray(app)?;
            let handle = app.handle().clone();
            let controller = app.state::<ControllerManager>();
            controller.set_app_handle(handle.clone());
            let script_runtime = app.state::<ScriptRuntime>();
            script_runtime.set_app_handle(handle.clone());

            // 启动后台定时任务调度引擎心跳循环
            scheduler::queue::start_scheduler_loop(handle.clone());

            // 启动 Forza 遥测 UDP 接收循环和转发 loop
            fh6_telemetry::init_telemetry_loop(handle.clone(), telemetry_state.clone());

            *log_handle.lock() = Some(handle.clone());

            #[cfg(target_os = "windows")]
            {
                let ocr_handle = handle.clone();
                let config_manager = app.state::<AppConfigManager>();
                let ocr_engine = config_manager.get().ocr_engine;
                std::thread::spawn(move || {
                    if let Err(e) = script_engine::ocr::preheat_ocr_engine(&ocr_handle, &ocr_engine)
                    {
                        tracing::warn!(target: "ocr", error = %e, "OCR engine preheat skipped");
                    }
                });
            }

            // 启动后台 CPU 和内存资源监控，并通过 Tauri 事件推送给前端
            system::resources::start_resource_monitor(handle.clone());

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
            controller_cmd::controller_reconnect_vigem,
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
            script_cmd::script_debug_execute,
            script_cmd::script_debug_resume,
            script_cmd::script_debug_step,
            script_cmd::script_debug_stop,
            script_cmd::script_stop,
            script_cmd::script_list,
            script_cmd::script_get,
            script_cmd::script_update,
            script_cmd::script_rename,
            script_cmd::script_delete,
            scheduler_cmd::scheduler_create_task,
            scheduler_cmd::scheduler_update_task,
            scheduler_cmd::scheduler_remove_task,
            scheduler_cmd::scheduler_toggle_task,
            scheduler_cmd::scheduler_list,
            scheduler_cmd::scheduler_execute_sequence,
            scheduler_cmd::scheduler_stop_sequence,
            config_cmd::config_get,
            config_cmd::config_set,
            config_cmd::open_ocr_viewfinder,
            config_cmd::save_ocr_region,
            config_cmd::run_ocr,
            config_cmd::run_ocr_detailed,
            config_cmd::preheat_ocr,
            config_cmd::export_backup_data,
            config_cmd::import_backup_data,
            log_cmd::log_query,
            log_cmd::log_export,
            notify::send_aggregated_notification,
            injector_cmd::get_injectable_processes,
            injector_cmd::get_injected_processes,
            injector_cmd::inject_focus_hook,
            injector_cmd::unload_focus_hook,
            injector_cmd::check_is_admin,
            injector_cmd::add_defender_exclusion,
            fh6_telemetry::commands::get_sessions,
            fh6_telemetry::commands::get_session_packets,
            fh6_telemetry::commands::get_session_laps,
            fh6_telemetry::commands::delete_session,
            fh6_telemetry::commands::clear_all_sessions,
            fh6_telemetry::commands::rename_session,
            fh6_telemetry::commands::set_session_bookmark,
            fh6_telemetry::commands::get_settings,
            fh6_telemetry::commands::save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
