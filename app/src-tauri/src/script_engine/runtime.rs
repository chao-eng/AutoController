use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::AppHandle;
use chrono::Utc;

use super::types::*;
use super::execution::Execution;
use super::sequence::SequenceExecution;
use crate::controller::ControllerManager;
use crate::persistence::DataDir;

pub struct ScriptRuntime {
    pub(super) scripts: Arc<Mutex<HashMap<String, Script>>>,
    pub(super) executions: Arc<Mutex<HashMap<String, Execution>>>,
    pub(super) sequence_executions: Arc<Mutex<HashMap<String, SequenceExecution>>>,
    pub(super) data_dir: Arc<DataDir>,
    pub(super) controller: Arc<ControllerManager>,
    pub(super) app_handle: Arc<Mutex<Option<AppHandle>>>,
}

unsafe impl Send for ScriptRuntime {}
unsafe impl Sync for ScriptRuntime {}

impl Clone for ScriptRuntime {
    fn clone(&self) -> Self {
        Self {
            scripts: self.scripts.clone(),
            executions: self.executions.clone(),
            sequence_executions: self.sequence_executions.clone(),
            data_dir: self.data_dir.clone(),
            controller: self.controller.clone(),
            app_handle: self.app_handle.clone(),
        }
    }
}

impl ScriptRuntime {
    pub fn new() -> Self {
        let data_dir = Arc::new(DataDir::new());
        let scripts = match data_dir.load::<HashMap<String, Script>>("scripts") {
            Some(data) => Arc::new(Mutex::new(data)),
            None => Arc::new(Mutex::new(HashMap::new())),
        };
        let controller = Arc::new(ControllerManager::new());

        Self {
            scripts,
            executions: Arc::new(Mutex::new(HashMap::new())),
            sequence_executions: Arc::new(Mutex::new(HashMap::new())),
            data_dir,
            controller,
            app_handle: Arc::new(Mutex::new(None)),
        }
    }

    pub fn with_controller(controller: ControllerManager) -> Self {
        let data_dir = Arc::new(DataDir::new());
        let scripts = match data_dir.load::<HashMap<String, Script>>("scripts") {
            Some(data) => Arc::new(Mutex::new(data)),
            None => Arc::new(Mutex::new(HashMap::new())),
        };

        Self {
            scripts,
            executions: Arc::new(Mutex::new(HashMap::new())),
            sequence_executions: Arc::new(Mutex::new(HashMap::new())),
            data_dir,
            controller: Arc::new(controller),
            app_handle: Arc::new(Mutex::new(None)),
        }
    }

    pub fn set_app_handle(&self, handle: AppHandle) {
        let mut app_handle = self.app_handle.lock();
        *app_handle = Some(handle);
    }

    fn persist(&self) {
        let scripts = self.scripts.lock();
        if let Err(e) = self.data_dir.save("scripts", &*scripts) {
            tracing::warn!(error = %e, "脚本数据持久化失败");
        }
    }

    pub fn create_script(&self, name: &str, code: &str) -> Script {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();
        let script = Script {
            id: id.clone(),
            name: name.to_string(),
            code: code.to_string(),
            created_at: now,
            updated_at: now,
        };
        let mut scripts = self.scripts.lock();
        scripts.insert(id, script.clone());
        drop(scripts);
        self.persist();
        tracing::info!(script_id = %script.id, name, "脚本已创建");
        script
    }

    pub fn update_script(&self, id: &str, code: &str) -> Result<Script, String> {
        let mut scripts = self.scripts.lock();
        if let Some(script) = scripts.get_mut(id) {
            script.code = code.to_string();
            script.updated_at = Utc::now();
            let updated = script.clone();
            drop(scripts);
            self.persist();
            Ok(updated)
        } else {
            Err(format!("脚本不存在: {}", id))
        }
    }

    pub fn rename_script(&self, id: &str, new_name: &str) -> Result<Script, String> {
        let mut scripts = self.scripts.lock();
        if let Some(script) = scripts.get_mut(id) {
            script.name = new_name.to_string();
            script.updated_at = Utc::now();
            let updated = script.clone();
            drop(scripts);
            self.persist();
            Ok(updated)
        } else {
            Err(format!("脚本不存在: {}", id))
        }
    }

    pub fn list_scripts(&self) -> Vec<ScriptMeta> {
        let scripts = self.scripts.lock();
        scripts
            .values()
            .map(|s| ScriptMeta {
                id: s.id.clone(),
                name: s.name.clone(),
                created_at: s.created_at,
                updated_at: s.updated_at,
            })
            .collect()
    }

    pub fn get_script(&self, id: &str) -> Option<Script> {
        let scripts = self.scripts.lock();
        scripts.get(id).cloned()
    }

    pub fn delete_script(&self, id: &str) -> Result<(), String> {
        let mut scripts = self.scripts.lock();
        if scripts.remove(id).is_some() {
            drop(scripts);
            self.persist();
            Ok(())
        } else {
            Err(format!("脚本不存在: {}", id))
        }
    }
}
