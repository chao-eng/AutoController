use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::str::FromStr;
use chrono::{DateTime, Utc, NaiveTime, Duration};
use cron::Schedule;
use tauri::Manager;

use super::types::*;
use crate::persistence::DataDir;
use crate::controller::ControllerManager;
use crate::macro_engine::MacroPlayer;
use crate::script_engine::ScriptRuntime;

/// 核心辅助：计算下一次执行的时间
pub fn calculate_next_run(schedule: &ScheduleType, last_run: Option<DateTime<Utc>>) -> Option<DateTime<Utc>> {
    let now = Utc::now();
    match schedule {
        ScheduleType::Once(dt) => {
            if last_run.is_some() {
                None // 已执行过
            } else if *dt > now {
                Some(*dt)
            } else {
                None // 已过期
            }
        }
        ScheduleType::Daily { time } => {
            // 解析 "HH:MM:SS" 或 "HH:MM"
            let time_parts: Vec<&str> = time.split(':').collect();
            if time_parts.is_empty() { return None; }
            let h = time_parts.get(0).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
            let m = time_parts.get(1).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
            let s = time_parts.get(2).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
            
            let local_now = chrono::Local::now();
            let naive_time = NaiveTime::from_hms_opt(h, m, s)?;
            let naive_date = local_now.date_naive();
            
            let mut next_local = naive_date.and_time(naive_time).and_local_timezone(chrono::Local).single()?;
            if next_local <= local_now {
                // 今天的时间已过，安排在明天
                next_local = next_local + Duration::days(1);
            }
            Some(next_local.with_timezone(&Utc))
        }
        ScheduleType::Interval { duration_ms } => {
            let last = last_run.unwrap_or(now);
            Some(last + Duration::milliseconds(*duration_ms as i64))
        }
        ScheduleType::Cron { expression } => {
            if let Ok(schedule) = Schedule::from_str(expression) {
                // 计算当前时间之后的下一次执行时刻
                schedule.upcoming(Utc).next()
            } else {
                tracing::warn!("无效的 Cron 表达式: {}", expression);
                None
            }
        }
        ScheduleType::Manual => None,
    }
}

pub struct TaskQueue {
    tasks: Arc<Mutex<HashMap<String, ScheduledTask>>>,
    data_dir: Arc<DataDir>,
}

impl TaskQueue {
    pub fn new() -> Self {
        let data_dir = Arc::new(DataDir::new());
        let tasks = match data_dir.load::<HashMap<String, ScheduledTask>>("tasks") {
            Some(mut data) => {
                // 启动时自动为启用的任务修复/初始化 next_run 属性，确保防丢失
                for task in data.values_mut() {
                    if task.enabled && task.next_run.is_none() {
                        task.next_run = calculate_next_run(&task.schedule, task.last_run);
                    }
                }
                Arc::new(Mutex::new(data))
            }
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

    pub fn add_task(&self, mut task: ScheduledTask) -> Result<(), String> {
        let mut tasks = self.tasks.lock();
        if tasks.contains_key(&task.id) {
            return Err(format!("任务已存在: {}", task.id));
        }
        // 初始化计算下一次运行时间
        task.next_run = calculate_next_run(&task.schedule, None);
        tracing::info!(task_id = %task.id, name = %task.name, "定时任务已创建并就绪");
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
            if enabled {
                task.next_run = calculate_next_run(&task.schedule, task.last_run);
            } else {
                task.next_run = None;
            }
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

    pub fn get_task(&self, task_id: &str) -> Option<ScheduledTask> {
        let tasks = self.tasks.lock();
        tasks.get(task_id).cloned()
    }
}

/// 后台智能任务调度引擎主循环线程
pub fn start_scheduler_loop(app_handle: tauri::AppHandle) {
    std::thread::spawn(move || {
        tracing::info!("AutoController 后台定时任务调度引擎已启动");
        loop {
            // 每秒心跳轮询一次，保证高精度与低功耗
            std::thread::sleep(std::time::Duration::from_millis(1000));
            
            let queue = match app_handle.try_state::<TaskQueue>() {
                Some(q) => q,
                None => continue,
            };
            
            let now = Utc::now();
            let mut tasks_to_run = Vec::new();
            
            // 1. 扫描当前需要执行的任务列表
            {
                let tasks = queue.tasks.lock();
                for task in tasks.values() {
                    if task.enabled {
                        if let Some(next) = task.next_run {
                            if next <= now {
                                tasks_to_run.push((task.id.clone(), task.action.clone(), task.priority));
                            }
                        }
                    }
                }
            }
            
            if tasks_to_run.is_empty() {
                continue;
            }
            
            // 2. 依据任务优先级进行高优先级抢占排序（优先级高的先执行）
            tasks_to_run.sort_by(|a, b| b.2.cmp(&a.2));
            
            // 3. 多线程异步非阻塞分发执行任务，防止耗时操作拖慢主轮询心跳
            for (task_id, action, _) in tasks_to_run {
                tracing::info!(task_id = %task_id, "触发定时调度任务");
                
                let handle_clone = app_handle.clone();
                let tid = task_id.clone();
                
                std::thread::spawn(move || {
                    let result = match &action {
                        TaskAction::PlayMacro { macro_id, speed, loop_count } => {
                            if let (Some(player), Some(recorder), Some(controller)) = (
                                handle_clone.try_state::<MacroPlayer>(),
                                handle_clone.try_state::<crate::macro_engine::MacroRecorder>(),
                                handle_clone.try_state::<ControllerManager>(),
                            ) {
                                if let Some(mac) = recorder.get_macro(macro_id) {
                                    player.start_playback(controller.inner().clone(), mac, *speed, *loop_count).map(|_| ())
                                } else {
                                    Err("指定手柄宏已不存在".to_string())
                                }
                            } else {
                                Err("手柄模拟基础设施未就绪".to_string())
                            }
                        }
                        TaskAction::ExecuteScript { script_id } => {
                            if let Some(runtime) = handle_clone.try_state::<ScriptRuntime>() {
                                runtime.execute_script(script_id).map(|_| ())
                            } else {
                                Err("脚本自动化引擎未就绪".to_string())
                            }
                        }
                        TaskAction::ExecuteSequence { steps, task_loop_count } => {
                            if let Some(runtime) = handle_clone.try_state::<ScriptRuntime>() {
                                runtime.execute_sequence(&tid, steps.clone(), *task_loop_count)
                            } else {
                                Err("脚本引擎未就绪".to_string())
                            }
                        }
                    };
                    
                    if let Err(e) = result {
                        tracing::error!(task_id = %tid, error = %e, "定时调度任务执行失败");
                    } else {
                        tracing::info!(task_id = %tid, "定时调度任务执行成功");
                    }
                });
                
                // 4. 更新调度任务状态并计算下一次运行时间
                {
                    let mut tasks = queue.tasks.lock();
                    if let Some(task) = tasks.get_mut(&task_id) {
                        task.last_run = Some(now);
                        // Once 类型的单次定时任务，执行后自动下线
                        if let ScheduleType::Once(_) = task.schedule {
                            task.enabled = false;
                            task.next_run = None;
                        } else {
                            task.next_run = calculate_next_run(&task.schedule, Some(now));
                        }
                    }
                }
                queue.persist();
            }
        }
    });
}
