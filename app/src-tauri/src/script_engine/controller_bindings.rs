use std::sync::Arc;
use parking_lot::Mutex;
use tauri::{AppHandle, Emitter};
use rhai::Engine;
use crate::controller::ControllerManager;
use super::bindings::{parse_axis, parse_button, parse_trigger};
use super::types::ScriptLineChangeEvent;

pub fn register_controller_bindings(
    engine: &mut Engine,
    controller: Arc<ControllerManager>,
    default_device: Arc<Mutex<String>>,
    app_handle: Arc<Mutex<Option<AppHandle>>>,
    execution_id: String,
    script_id: String,
) {
    // set_default_device(device_id)
    let def_device = default_device.clone();
    engine.register_fn("set_default_device", move |device_id: i64| {
        let mut d = def_device.lock();
        *d = device_id.to_string();
    });

    let def_device = default_device.clone();
    engine.register_fn("set_default_device", move |device_id: &str| {
        let mut d = def_device.lock();
        *d = device_id.to_string();
    });

    // press(device_id, button) & press(button)
    let ctrl = controller.clone();
    engine.register_fn("press", move |device_id: i64, btn: &str| {
        if let Some(b) = parse_button(btn) {
            if let Err(e) = ctrl.set_button(&device_id.to_string(), b, true) {
                tracing::warn!(target: "script", error = %e, "press 失败");
            }
        } else {
            tracing::warn!(target: "script", "未知按键: {}", btn);
        }
    });

    let ctrl = controller.clone();
    engine.register_fn("press", move |device_id: &str, btn: &str| {
        if let Some(b) = parse_button(btn) {
            if let Err(e) = ctrl.set_button(device_id, b, true) {
                tracing::warn!(target: "script", error = %e, "press 失败");
            }
        } else {
            tracing::warn!(target: "script", "未知按键: {}", btn);
        }
    });

    let ctrl = controller.clone();
    let def_device = default_device.clone();
    let handle_press = app_handle.clone();
    let eid_press = execution_id.clone();
    let sid_press = script_id.clone();
    engine.register_fn(
        "press",
        move |context: rhai::NativeCallContext, btn: &str| {
            {
                let handle_guard = handle_press.lock();
                if let Some(ref handle) = *handle_guard {
                    if let Some(line) = context.call_position().line() {
                        let _ = handle.emit(
                            "script-line-change",
                            ScriptLineChangeEvent {
                                execution_id: eid_press.clone(),
                                script_id: sid_press.clone(),
                                line,
                            },
                        );
                    }
                }
            }
            let dev = def_device.lock().clone();
            if let Some(b) = parse_button(btn) {
                if let Err(e) = ctrl.set_button(&dev, b, true) {
                    tracing::warn!(target: "script", error = %e, "press 失败");
                }
            } else {
                tracing::warn!(target: "script", "未知按键: {}", btn);
            }
        },
    );

    // release(device_id, button) & release(button)
    let ctrl = controller.clone();
    engine.register_fn("release", move |device_id: i64, btn: &str| {
        if let Some(b) = parse_button(btn) {
            if let Err(e) = ctrl.set_button(&device_id.to_string(), b, false) {
                tracing::warn!(target: "script", error = %e, "release 失败");
            }
        } else {
            tracing::warn!(target: "script", "未知按键: {}", btn);
        }
    });

    let ctrl = controller.clone();
    engine.register_fn("release", move |device_id: &str, btn: &str| {
        if let Some(b) = parse_button(btn) {
            if let Err(e) = ctrl.set_button(device_id, b, false) {
                tracing::warn!(target: "script", error = %e, "release 失败");
            }
        } else {
            tracing::warn!(target: "script", "未知按键: {}", btn);
        }
    });

    let ctrl = controller.clone();
    let def_device = default_device.clone();
    let handle_release = app_handle.clone();
    let eid_release = execution_id.clone();
    let sid_release = script_id.clone();
    engine.register_fn(
        "release",
        move |context: rhai::NativeCallContext, btn: &str| {
            {
                let handle_guard = handle_release.lock();
                if let Some(ref handle) = *handle_guard {
                    if let Some(line) = context.call_position().line() {
                        let _ = handle.emit(
                            "script-line-change",
                            ScriptLineChangeEvent {
                                execution_id: eid_release.clone(),
                                script_id: sid_release.clone(),
                                line,
                            },
                        );
                    }
                }
            }
            let dev = def_device.lock().clone();
            if let Some(b) = parse_button(btn) {
                if let Err(e) = ctrl.set_button(&dev, b, false) {
                    tracing::warn!(target: "script", error = %e, "release 失败");
                }
            } else {
                tracing::warn!(target: "script", "未知按键: {}", btn);
            }
        },
    );

    // set_thumb(device_id, axis, value) & set_thumb(axis, value)
    let ctrl = controller.clone();
    engine.register_fn("set_thumb", move |device_id: i64, axis: &str, val: f64| {
        if let Some(a) = parse_axis(axis) {
            if let Err(e) = ctrl.set_thumb(&device_id.to_string(), a, val as f32) {
                tracing::warn!(target: "script", error = %e, "set_thumb 失败");
            }
        } else {
            tracing::warn!(target: "script", "未知摇杆轴: {}", axis);
        }
    });

    let ctrl = controller.clone();
    engine.register_fn("set_thumb", move |device_id: i64, axis: &str, val: i64| {
        if let Some(a) = parse_axis(axis) {
            if let Err(e) = ctrl.set_thumb(&device_id.to_string(), a, val as f32) {
                tracing::warn!(target: "script", error = %e, "set_thumb 失败");
            }
        } else {
            tracing::warn!(target: "script", "未知摇杆轴: {}", axis);
        }
    });

    let ctrl = controller.clone();
    engine.register_fn("set_thumb", move |device_id: &str, axis: &str, val: f64| {
        if let Some(a) = parse_axis(axis) {
            if let Err(e) = ctrl.set_thumb(device_id, a, val as f32) {
                tracing::warn!(target: "script", error = %e, "set_thumb 失败");
            }
        } else {
            tracing::warn!(target: "script", "未知摇杆轴: {}", axis);
        }
    });

    let ctrl = controller.clone();
    engine.register_fn("set_thumb", move |device_id: &str, axis: &str, val: i64| {
        if let Some(a) = parse_axis(axis) {
            if let Err(e) = ctrl.set_thumb(device_id, a, val as f32) {
                tracing::warn!(target: "script", error = %e, "set_thumb 失败");
            }
        } else {
            tracing::warn!(target: "script", "未知摇杆轴: {}", axis);
        }
    });

    let ctrl = controller.clone();
    let def_device = default_device.clone();
    let handle_thumb_f = app_handle.clone();
    let eid_thumb_f = execution_id.clone();
    let sid_thumb_f = script_id.clone();
    engine.register_fn(
        "set_thumb",
        move |context: rhai::NativeCallContext, axis: &str, val: f64| {
            {
                let handle_guard = handle_thumb_f.lock();
                if let Some(ref handle) = *handle_guard {
                    if let Some(line) = context.call_position().line() {
                        let _ = handle.emit(
                            "script-line-change",
                            ScriptLineChangeEvent {
                                execution_id: eid_thumb_f.clone(),
                                script_id: sid_thumb_f.clone(),
                                line,
                            },
                        );
                    }
                }
            }
            let dev = def_device.lock().clone();
            if let Some(a) = parse_axis(axis) {
                if let Err(e) = ctrl.set_thumb(&dev, a, val as f32) {
                    tracing::warn!(target: "script", error = %e, "set_thumb 失败");
                }
            } else {
                tracing::warn!(target: "script", "未知摇杆轴: {}", axis);
            }
        },
    );

    let ctrl = controller.clone();
    let def_device = default_device.clone();
    let handle_thumb_i = app_handle.clone();
    let eid_thumb_i = execution_id.clone();
    let sid_thumb_i = script_id.clone();
    engine.register_fn(
        "set_thumb",
        move |context: rhai::NativeCallContext, axis: &str, val: i64| {
            {
                let handle_guard = handle_thumb_i.lock();
                if let Some(ref handle) = *handle_guard {
                    if let Some(line) = context.call_position().line() {
                        let _ = handle.emit(
                            "script-line-change",
                            ScriptLineChangeEvent {
                                execution_id: eid_thumb_i.clone(),
                                script_id: sid_thumb_i.clone(),
                                line,
                            },
                        );
                    }
                }
            }
            let dev = def_device.lock().clone();
            if let Some(a) = parse_axis(axis) {
                if let Err(e) = ctrl.set_thumb(&dev, a, val as f32) {
                    tracing::warn!(target: "script", error = %e, "set_thumb 失败");
                }
            } else {
                tracing::warn!(target: "script", "未知摇杆轴: {}", axis);
            }
        },
    );

    // set_trigger(device_id, side, value) & set_trigger(side, value)
    let ctrl = controller.clone();
    engine.register_fn(
        "set_trigger",
        move |device_id: i64, side: &str, val: f64| {
            if let Some(s) = parse_trigger(side) {
                if let Err(e) = ctrl.set_trigger(&device_id.to_string(), s, val as f32) {
                    tracing::warn!(target: "script", error = %e, "set_trigger 失败");
                }
            } else {
                tracing::warn!(target: "script", "未知扳机侧: {}", side);
            }
        },
    );

    let ctrl = controller.clone();
    engine.register_fn(
        "set_trigger",
        move |device_id: i64, side: &str, val: i64| {
            if let Some(s) = parse_trigger(side) {
                if let Err(e) = ctrl.set_trigger(&device_id.to_string(), s, val as f32) {
                    tracing::warn!(target: "script", error = %e, "set_trigger 失败");
                }
            } else {
                tracing::warn!(target: "script", "未知扳机侧: {}", side);
            }
        },
    );

    let ctrl = controller.clone();
    engine.register_fn(
        "set_trigger",
        move |device_id: &str, side: &str, val: f64| {
            if let Some(s) = parse_trigger(side) {
                if let Err(e) = ctrl.set_trigger(device_id, s, val as f32) {
                    tracing::warn!(target: "script", error = %e, "set_trigger 失败");
                }
            } else {
                tracing::warn!(target: "script", "未知扳机侧: {}", side);
            }
        },
    );

    let ctrl = controller.clone();
    engine.register_fn(
        "set_trigger",
        move |device_id: &str, side: &str, val: i64| {
            if let Some(s) = parse_trigger(side) {
                if let Err(e) = ctrl.set_trigger(device_id, s, val as f32) {
                    tracing::warn!(target: "script", error = %e, "set_trigger 失败");
                }
            } else {
                tracing::warn!(target: "script", "未知扳机侧: {}", side);
            }
        },
    );

    let ctrl = controller.clone();
    let def_device = default_device.clone();
    let handle_trig_f = app_handle.clone();
    let eid_trig_f = execution_id.clone();
    let sid_trig_f = script_id.clone();
    engine.register_fn(
        "set_trigger",
        move |context: rhai::NativeCallContext, side: &str, val: f64| {
            {
                let handle_guard = handle_trig_f.lock();
                if let Some(ref handle) = *handle_guard {
                    if let Some(line) = context.call_position().line() {
                        let _ = handle.emit(
                            "script-line-change",
                            ScriptLineChangeEvent {
                                execution_id: eid_trig_f.clone(),
                                script_id: sid_trig_f.clone(),
                                line,
                            },
                        );
                    }
                }
            }
            let dev = def_device.lock().clone();
            if let Some(s) = parse_trigger(side) {
                if let Err(e) = ctrl.set_trigger(&dev, s, val as f32) {
                    tracing::warn!(target: "script", error = %e, "set_trigger 失败");
                }
            } else {
                tracing::warn!(target: "script", "未知扳机侧: {}", side);
            }
        },
    );

    let ctrl = controller.clone();
    let def_device = default_device.clone();
    let handle_trig_i = app_handle.clone();
    let eid_trig_i = execution_id.clone();
    let sid_trig_i = script_id.clone();
    engine.register_fn(
        "set_trigger",
        move |context: rhai::NativeCallContext, side: &str, val: i64| {
            {
                let handle_guard = handle_trig_i.lock();
                if let Some(ref handle) = *handle_guard {
                    if let Some(line) = context.call_position().line() {
                        let _ = handle.emit(
                            "script-line-change",
                            ScriptLineChangeEvent {
                                execution_id: eid_trig_i.clone(),
                                script_id: sid_trig_i.clone(),
                                line,
                            },
                        );
                    }
                }
            }
            let dev = def_device.lock().clone();
            if let Some(s) = parse_trigger(side) {
                if let Err(e) = ctrl.set_trigger(&dev, s, val as f32) {
                    tracing::warn!(target: "script", error = %e, "set_trigger 失败");
                }
            } else {
                tracing::warn!(target: "script", "未知扳机侧: {}", side);
            }
        },
    );
}
