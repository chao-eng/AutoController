use std::sync::Arc;
use parking_lot::Mutex;
use tauri::{AppHandle, Emitter};
use rhai::Engine;
use super::types::ScriptLineChangeEvent;


pub fn register_ocr_bindings(
    engine: &mut Engine,
    app_handle: Arc<Mutex<Option<AppHandle>>>,
    execution_id: String,
    script_id: String,
) {
    let handle_ocr_params = app_handle.clone();
    let eid_ocr_params = execution_id.clone();
    let sid_ocr_params = script_id.clone();
    engine.register_fn(
        "ocr",
        move |context: rhai::NativeCallContext, x: i64, y: i64, w: i64, h: i64| -> String {
            let (ocr_engine, ocr_profile, paddleocr_url, opt_handle) = {
                let handle_guard = handle_ocr_params.lock();
                if let Some(ref handle) = *handle_guard {
                    if let Some(line) = context.call_position().line() {
                        let _ = handle.emit(
                            "script-line-change",
                            ScriptLineChangeEvent {
                                execution_id: eid_ocr_params.clone(),
                                script_id: sid_ocr_params.clone(),
                                line,
                            },
                        );
                    }
                    use tauri::Manager;
                    let config_mgr = handle.state::<crate::config::AppConfigManager>();
                    let config = config_mgr.get();
                    (
                        config.ocr_engine.clone(),
                        config.ocr_profile.clone(),
                        config.paddleocr_url.clone(),
                        Some(handle.clone()),
                    )
                } else {
                    (
                        "paddleocr".to_string(),
                        "balanced".to_string(),
                        "http://127.0.0.1:8050/ocr".to_string(),
                        None,
                    )
                }
            };

            match crate::script_engine::ocr::ocr_region_sync(
                x as i32,
                y as i32,
                w as i32,
                h as i32,
                &ocr_engine,
                &ocr_profile,
                &paddleocr_url,
                opt_handle.as_ref(),
            ) {
                Ok(text) => text,
                Err(e) => {
                    tracing::error!(target: "script", "OCR 识别出错: {}", e);
                    String::new()
                }
            }
        },
    );

    let handle_ocr_def = app_handle.clone();
    let eid_ocr_def = execution_id.clone();
    let sid_ocr_def = script_id.clone();
    engine.register_fn("ocr", move |context: rhai::NativeCallContext| -> String {
        {
            let handle_guard = handle_ocr_def.lock();
            if let Some(ref handle) = *handle_guard {
                if let Some(line) = context.call_position().line() {
                    let _ = handle.emit("script-line-change", ScriptLineChangeEvent {
                        execution_id: eid_ocr_def.clone(),
                        script_id: sid_ocr_def.clone(),
                        line,
                    });
                }
                use tauri::Manager;
                let config_mgr = handle.state::<crate::config::AppConfigManager>();
                let config = config_mgr.get();

                // 优先使用 ocr_regions 的第一个作为默认识别区，其次使用老字段 ocr_region 兼容
                let target_region = if !config.ocr_regions.is_empty() {
                    Some(config.ocr_regions[0].clone())
                } else {
                    config.ocr_region.clone()
                };

                if let Some(region) = target_region {
                    let ocr_engine = config.ocr_engine.clone();
                    let ocr_profile = config.ocr_profile.clone();
                    let paddleocr_url = config.paddleocr_url.clone();
                    return match crate::script_engine::ocr::ocr_region_sync(region.x, region.y, region.w, region.h, &ocr_engine, &ocr_profile, &paddleocr_url, Some(handle)) {
                        Ok(text) => text,
                        Err(e) => {
                            tracing::error!(target: "script", "OCR 默认区域 #1 识别出错: {}", e);
                            String::new()
                        }
                    };
                } else {
                    tracing::warn!(target: "script", "OCR 默认区域 #1 尚未配置，请在前端配置或传入坐标");
                    return String::new();
                }
            }
        }
        tracing::warn!(target: "script", "ocr() 无参调用失败：AppHandle 尚未初始化");
        String::new()
    });

    let handle_ocr_idx = app_handle.clone();
    let eid_ocr_idx = execution_id.clone();
    let sid_ocr_idx = script_id.clone();
    engine.register_fn("ocr", move |context: rhai::NativeCallContext, index: i64| -> String {
        {
            let handle_guard = handle_ocr_idx.lock();
            if let Some(ref handle) = *handle_guard {
                if let Some(line) = context.call_position().line() {
                    let _ = handle.emit("script-line-change", ScriptLineChangeEvent {
                        execution_id: eid_ocr_idx.clone(),
                        script_id: sid_ocr_idx.clone(),
                        line,
                    });
                }
                if index <= 0 {
                    tracing::error!(target: "script", "ocr(index) 序号错误：序号必须从 1 开始（传入为 {}）", index);
                    return String::new();
                }
                use tauri::Manager;
                let config_mgr = handle.state::<crate::config::AppConfigManager>();
                let config = config_mgr.get();
                let u_idx = (index - 1) as usize;

                if u_idx < config.ocr_regions.len() {
                    let region = &config.ocr_regions[u_idx];
                    let ocr_engine = config.ocr_engine.clone();
                    let ocr_profile = config.ocr_profile.clone();
                    let paddleocr_url = config.paddleocr_url.clone();
                    return match crate::script_engine::ocr::ocr_region_sync(region.x, region.y, region.w, region.h, &ocr_engine, &ocr_profile, &paddleocr_url, Some(handle)) {
                        Ok(text) => text,
                        Err(e) => {
                            tracing::error!(target: "script", "OCR 区域 #{} 识别出错: {}", index, e);
                            String::new()
                        }
                    };
                } else {
                    tracing::warn!(target: "script", "OCR 区域 #{} 尚未配置，当前已配置区域数: {}", index, config.ocr_regions.len());
                    return String::new();
                }
            }
        }
        tracing::warn!(target: "script", "ocr(index) 调用失败：AppHandle 尚未初始化");
        String::new()
    });
}
