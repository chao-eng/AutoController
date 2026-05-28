use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

use super::types::*;
use crate::persistence::DataDir;

pub struct TaskQueue {
    tasks: Arc<Mutex<HashMap<String, ScheduledTask>>>,
    data_dir: Arc<DataDir>,
}

impl TaskQueue {
    pub fn new() -> Self {
        let data_dir = Arc::new(DataDir::new());
        let tasks = match data_dir.load::<HashMap<String, ScheduledTask>>("tasks") {
            Some(data) => Arc::new(Mutex::new(data)),
            None => Arc::new(Mutex::new(HashMap::new())),
        };
        Self { tasks, data_dir }
    }

    fn persist(&self) {
        let tasks = self.tasks.lock();
        if let Err(e) = self.data_dir.save("tasks", &*tasks) {
            tracing::warn!(error = %e, "任务数据持久化失败");
        }
    }

    pub fn add_task(&self, task: ScheduledTask) -> Result<(), String> {
        let mut tasks = self.tasks.lock();
        if tasks.contains_key(&task.id) {
            return Err(format!("任务已存在: {}", task.id));
        }
        tracing::info!(task_id = %task.id, name = %task.name, "定时任务已创建");
        tasks.insert(task.id.clone(), task);
        drop(tasks);
        self.persist();
        Ok(())
    }

    pub fn remove_task(&self, task_id: &str) -> Result<(), String> {
        let mut tasks = self.tasks.lock();
        if tasks.remove(task_id).is_some() {
            drop(tasks);
            self.persist();
            tracing::info!(task_id = %task_id, "定时任务已删除");
            Ok(())
        } else {
            Err(format!("任务不存在: {}", task_id))
        }
    }

    pub fn toggle_task(&self, task_id: &str, enabled: bool) -> Result<(), String> {
        let mut tasks = self.tasks.lock();
        if let Some(task) = tasks.get_mut(task_id) {
            task.enabled = enabled;
            drop(tasks);
            self.persist();
            tracing::info!(task_id = %task_id, enabled, "任务状态已切换");
            Ok(())
        } else {
            Err(format!("任务不存在: {}", task_id))
        }
    }

    pub fn list_tasks(&self) -> Vec<ScheduledTask> {
        let tasks = self.tasks.lock();
        let mut list: Vec<_> = tasks.values().cloned().collect();
        list.sort_by(|a, b| b.priority.cmp(&a.priority));
        list
    }

    #[allow(dead_code)]
    pub fn get_task(&self, task_id: &str) -> Option<ScheduledTask> {
        let tasks = self.tasks.lock();
        tasks.get(task_id).cloned()
    }
}
