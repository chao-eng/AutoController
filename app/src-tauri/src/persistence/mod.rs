use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use tracing;

#[derive(Clone)]
pub struct DataDir {
    base: PathBuf,
}

impl DataDir {
    pub fn new() -> Self {
        let base = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("autocontroller");
        Self { base }
    }

    pub fn ensure_dir(&self) -> Result<()> {
        fs::create_dir_all(&self.base)?;
        Ok(())
    }

    pub fn path(&self, name: &str) -> PathBuf {
        self.base.join(format!("{}.json", name))
    }

    pub fn save<T: serde::Serialize>(&self, name: &str, data: &T) -> Result<()> {
        self.ensure_dir()?;
        let path = self.path(name);
        let json = serde_json::to_string_pretty(data)?;
        fs::write(&path, &json)?;
        tracing::debug!(path = %path.display(), "数据已保存");
        Ok(())
    }

    pub fn load<T: serde::de::DeserializeOwned>(&self, name: &str) -> Option<T> {
        let path = self.path(name);
        if !path.exists() {
            return None;
        }
        match fs::read_to_string(&path) {
            Ok(json) => match serde_json::from_str::<T>(&json) {
                Ok(data) => {
                    tracing::debug!(path = %path.display(), "数据已加载");
                    Some(data)
                }
                Err(e) => {
                    tracing::warn!(path = %path.display(), error = %e, "数据解析失败");
                    None
                }
            },
            Err(e) => {
                tracing::warn!(path = %path.display(), error = %e, "数据读取失败");
                None
            }
        }
    }
}
