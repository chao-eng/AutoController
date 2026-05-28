use crate::logger::LogQuery;

#[tauri::command]
pub fn log_query(query: LogQuery) -> Vec<crate::logger::LogEntry> {
    let _ = query;
    Vec::new()
}

#[tauri::command]
pub fn log_export(format: String, query: LogQuery) -> Result<String, String> {
    let _ = (format, query);
    Err("日志导出功能尚未实现".to_string())
}
