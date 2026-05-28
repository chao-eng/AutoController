use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::persistence::DataDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfig {
    pub id: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameProfile {
    pub id: String,
    pub name: String,
    pub game_process: String,
    pub macros: Vec<String>,
    pub scripts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrRegion {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub devices: Vec<DeviceConfig>,
    pub profiles: Vec<GameProfile>,
    pub active_profile: Option<String>,
    pub auto_start: bool,
    pub minimize_to_tray: bool,
    pub log_level: String,
    pub ocr_region: Option<OcrRegion>,
    #[serde(default)]
    pub ocr_regions: Vec<OcrRegion>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            devices: Vec::new(),
            profiles: Vec::new(),
            active_profile: None,
            auto_start: false,
            minimize_to_tray: true,
            log_level: "info".to_string(),
            ocr_region: None,
            ocr_regions: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub struct AppConfigManager {
    config: Arc<Mutex<AppConfig>>,
    data_dir: Arc<DataDir>,
}

impl AppConfigManager {
    pub fn new() -> Self {
        let data_dir = Arc::new(DataDir::new());
        let config = match data_dir.load::<AppConfig>("config") {
            Some(data) => Arc::new(Mutex::new(data)),
            None => Arc::new(Mutex::new(AppConfig::default())),
        };
        Self { config, data_dir }
    }

    fn persist(&self) {
        let config = self.config.lock();
        if let Err(e) = self.data_dir.save("config", &*config) {
            tracing::warn!(error = %e, "配置数据持久化失败");
        }
    }

    pub fn get(&self) -> AppConfig {
        self.config.lock().clone()
    }

    pub fn set(&self, new_config: AppConfig) {
        let mut config = self.config.lock();
        *config = new_config;
        drop(config);
        self.persist();
    }
}
