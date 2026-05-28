use crate::config::{AppConfig, AppConfigManager};

#[tauri::command]
pub fn config_get(manager: tauri::State<'_, AppConfigManager>) -> AppConfig {
    manager.get()
}

#[tauri::command]
pub fn config_set(
    manager: tauri::State<'_, AppConfigManager>,
    reload_handle: tauri::State<'_, tracing_subscriber::reload::Handle<tracing_subscriber::EnvFilter, tracing_subscriber::Registry>>,
    new_config: AppConfig,
) -> Result<(), String> {
    let new_level = new_config.log_level.clone();
    manager.set(new_config);

    // 动态更新全局日志过滤器级别
    if let Ok(new_filter) = tracing_subscriber::EnvFilter::try_new(format!("{},tao=error", new_level)) {
        if let Err(e) = reload_handle.reload(new_filter) {
            tracing::warn!(error = %e, "动态更新日志级别过滤失败");
        } else {
            tracing::info!("日志级别已动态更新为: {}", new_level);
        }
    }

    Ok(())
}
