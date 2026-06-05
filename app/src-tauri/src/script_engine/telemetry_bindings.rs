use rhai::Engine;
use crate::fh6_telemetry;

pub fn register_telemetry_bindings(engine: &mut Engine) {
    engine.register_fn("get_telemetry", || -> rhai::Map {
        let lock = fh6_telemetry::LAST_TELEMETRY.get_or_init(|| std::sync::RwLock::new(None));
        if let Ok(guard) = lock.read() {
            if let Some(ref state) = *guard {
                let mut map = rhai::Map::new();
                map.insert("car_name".into(), state.car_name.clone().into());
                map.insert("speed".into(), (state.speed as f64).into());
                map.insert("speed_kmh".into(), (state.speed_kmh as f64).into());
                map.insert("is_race_on".into(), state.is_race_on.into());
                map.insert("car_ordinal".into(), (state.car_ordinal as i64).into());
                map.insert("engine_max_rpm".into(), (state.engine_max_rpm as f64).into());
                map.insert("current_engine_rpm".into(), (state.current_engine_rpm as f64).into());
                map.insert("gear".into(), (state.gear as i64).into());
                map.insert("throttle".into(), (state.throttle as i64).into());
                map.insert("brake".into(), (state.brake as i64).into());
                map.insert("clutch".into(), (state.clutch as i64).into());
                map.insert("handbrake".into(), (state.handbrake as i64).into());
                map.insert("current_lap".into(), (state.current_lap as f64).into());
                map.insert("current_race_time".into(), (state.current_race_time as f64).into());
                map.insert("lap_number".into(), (state.lap_number as i64).into());
                map.insert("race_position".into(), (state.race_position as i64).into());
                return map;
            }
        }

        // Return a default/empty map if no telemetry packet has been received yet
        let mut map = rhai::Map::new();
        map.insert("car_name".into(), "".into());
        map.insert("speed".into(), 0.0.into());
        map.insert("speed_kmh".into(), 0.0.into());
        map.insert("is_race_on".into(), false.into());
        map.insert("car_ordinal".into(), 0.into());
        map.insert("engine_max_rpm".into(), 0.0.into());
        map.insert("current_engine_rpm".into(), 0.0.into());
        map.insert("gear".into(), 0.into());
        map.insert("throttle".into(), 0.into());
        map.insert("brake".into(), 0.into());
        map.insert("clutch".into(), 0.into());
        map.insert("handbrake".into(), 0.into());
        map.insert("current_lap".into(), 0.0.into());
        map.insert("current_race_time".into(), 0.0.into());
        map.insert("lap_number".into(), 0.into());
        map.insert("race_position".into(), 0.into());
        map
    });
}
