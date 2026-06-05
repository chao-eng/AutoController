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
use std::sync::RwLock;
use std::sync::OnceLock;
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TelemetryState {
    pub car_name: String,
    pub speed: f32,
    pub speed_kmh: f32,
    pub is_race_on: bool,
    pub car_ordinal: i32,
    pub engine_max_rpm: f32,
    pub current_engine_rpm: f32,
    pub gear: u8,
    pub throttle: u8,
    pub brake: u8,
    pub clutch: u8,
    pub handbrake: u8,
    pub current_lap: f32,
    pub current_race_time: f32,
    pub lap_number: u16,
    pub race_position: u8,
}

pub static LAST_TELEMETRY: OnceLock<RwLock<Option<TelemetryState>>> = OnceLock::new();

pub fn get_car_name(ordinal: i32) -> String {
    static CAR_MAP: OnceLock<HashMap<i32, String>> = OnceLock::new();
    let map = CAR_MAP.get_or_init(|| {
        let mut map = HashMap::new();
        // 加载旧版车辆数据 (legacy)
        if let Ok(legacy) = serde_json::from_str::<HashMap<String, String>>(include_str!("../../../src/fh6-tel/lib/car-ordinals.json")) {
            for (k, v) in legacy {
                if let Ok(ord) = k.parse::<i32>() {
                    map.insert(ord, v);
                }
            }
        }
        // 加载新版确认车辆数据 (优先覆盖)
        if let Ok(fh6) = serde_json::from_str::<HashMap<String, String>>(include_str!("../../../src/fh6-tel/lib/fh6-car-ordinals.json")) {
            for (k, v) in fh6 {
                if let Ok(ord) = k.parse::<i32>() {
                    map.insert(ord, v);
                }
            }
        }
        map
    });

    map.get(&ordinal).cloned().unwrap_or_else(|| format!("Car #{}", ordinal))
}

pub fn update_last_telemetry(pkt: &parser::TelemetryPacket) {
    let car_name = get_car_name(pkt.car_ordinal);
    let speed_kmh = pkt.speed_ms * 3.6;
    let state = TelemetryState {
        car_name,
        speed: pkt.speed_ms,
        speed_kmh,
        is_race_on: pkt.is_race_on,
        car_ordinal: pkt.car_ordinal,
        engine_max_rpm: pkt.engine_max_rpm,
        current_engine_rpm: pkt.current_engine_rpm,
        gear: pkt.gear,
        throttle: pkt.throttle,
        brake: pkt.brake,
        clutch: pkt.clutch,
        handbrake: pkt.handbrake,
        current_lap: pkt.current_lap,
        current_race_time: pkt.current_race_time,
        lap_number: pkt.lap_number,
        race_position: pkt.race_position,
    };

    let lock = LAST_TELEMETRY.get_or_init(|| RwLock::new(None));
    if let Ok(mut guard) = lock.write() {
        *guard = Some(state);
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_car_name() {
        // Test known ordinal from fh6-car-ordinals.json
        let name = get_car_name(1006);
        assert_eq!(name, "2005 Ferrari FXX");

        // Test unknown ordinal
        let name_unknown = get_car_name(999999);
        assert_eq!(name_unknown, "Car #999999");
    }

    #[test]
    fn test_update_last_telemetry() {
        let pkt = parser::TelemetryPacket {
            is_race_on: true,
            timestamp_ms: 100,
            engine_max_rpm: 8000.0,
            engine_idle_rpm: 1000.0,
            current_engine_rpm: 3000.0,
            accel_x: 0.0,
            accel_y: 0.0,
            accel_z: 0.0,
            vel_x: 0.0,
            vel_y: 0.0,
            vel_z: 0.0,
            position_x: 0.0,
            position_y: 0.0,
            position_z: 0.0,
            tire_slip_ratio_fl: 0.0,
            tire_slip_ratio_fr: 0.0,
            tire_slip_ratio_rl: 0.0,
            tire_slip_ratio_rr: 0.0,
            tire_slip_angle_fl: 0.0,
            tire_slip_angle_fr: 0.0,
            tire_slip_angle_rl: 0.0,
            tire_slip_angle_rr: 0.0,
            car_ordinal: 1006,
            car_class: 0,
            car_pi: 900,
            drivetrain_type: 0,
            speed_ms: 50.0, // 180 km/h
            power: 0.0,
            torque: 0.0,
            tire_temp_fl: 0.0,
            tire_temp_fr: 0.0,
            tire_temp_rl: 0.0,
            tire_temp_rr: 0.0,
            boost: 0.0,
            fuel: 0.0,
            distance_traveled: 0.0,
            best_lap: 0.0,
            last_lap: 0.0,
            current_lap: 12.5,
            current_race_time: 25.0,
            lap_number: 2,
            race_position: 3,
            throttle: 255,
            brake: 0,
            clutch: 0,
            handbrake: 0,
            gear: 4,
            steer: 0,
            yaw: 0.0,
            pitch: 0.0,
            roll: 0.0,
            suspension_fl: 0.0,
            suspension_fr: 0.0,
            suspension_rl: 0.0,
            suspension_rr: 0.0,
            tire_wear_fl: None,
            tire_wear_fr: None,
            tire_wear_rl: None,
            tire_wear_rr: None,
        };

        update_last_telemetry(&pkt);

        let lock = LAST_TELEMETRY.get().unwrap();
        let guard = lock.read().unwrap();
        let state = guard.as_ref().unwrap();

        assert_eq!(state.car_name, "2005 Ferrari FXX");
        assert_eq!(state.speed, 50.0);
        assert_eq!(state.speed_kmh, 180.0);
        assert_eq!(state.is_race_on, true);
        assert_eq!(state.car_ordinal, 1006);
        assert_eq!(state.gear, 4);
        assert_eq!(state.throttle, 255);
        assert_eq!(state.brake, 0);
        assert_eq!(state.current_lap, 12.5);
        assert_eq!(state.current_race_time, 25.0);
        assert_eq!(state.lap_number, 2);
        assert_eq!(state.race_position, 3);
    }
}
