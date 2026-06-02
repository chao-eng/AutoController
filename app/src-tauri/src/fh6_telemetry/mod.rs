pub mod api;
pub mod commands;
pub mod db;
pub mod event;
pub mod parser;
pub mod session;
pub mod settings;
pub mod udp;

use session::SessionManager;
use std::sync::Mutex;

pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
    pub session_manager: Mutex<SessionManager>,
    pub settings: Mutex<settings::Settings>,
}

/// Shared owner so the UDP writer and the request handlers share one DB mutex.
pub type Shared = std::sync::Arc<AppState>;

pub fn init_telemetry_loop(app_handle: tauri::AppHandle, state: Shared) {
    use tauri::Emitter;

    let port = state.settings.lock().unwrap().port;

    // The initial receiver is dropped; the forwarder task below calls tx.subscribe()
    // before the ingest loop starts sending, so no ticks are missed.
    let (tx, _rx) = tokio::sync::broadcast::channel::<event::ServerEvent>(256);

    // Forward broadcast events to the webview via the original event names.
    let mut rx = tx.subscribe();
    let handle = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        while let Ok(ev) = rx.recv().await {
            match ev {
                event::ServerEvent::Tick(pkt) => {
                    let _ = handle.emit("telemetry_tick", &pkt);
                }
                event::ServerEvent::BindFailed(msg) => {
                    let _ = handle.emit("udp_bind_failed", msg);
                }
                event::ServerEvent::SessionError(msg) => {
                    let _ = handle.emit("session_error", msg);
                }
            }
        }
    });

    // Ingest loop.
    let udp_state = state.clone();
    let udp_tx = tx.clone();
    tauri::async_runtime::spawn(async move {
        udp::run(udp_state, port, udp_tx).await;
    });
}
