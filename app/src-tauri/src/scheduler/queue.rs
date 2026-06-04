use chrono::{DateTime, Duration, NaiveTime, Utc};
use cron::Schedule;
use parking_lot::Mutex;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Semaphore;
use tokio::time::MissedTickBehavior;

use super::types::*;
use crate::controller::ControllerManager;
use crate::macro_engine::MacroPlayer;
use crate::persistence::DataDir;
use crate::script_engine::ScriptRuntime;

const SCHEDULER_TICK_MS: u64 = 1000;
const MAX_CONCURRENT_DISPATCHES: usize = 4;

/// 核心辅助：计算下一次执行的时间
pub fn calculate_next_run(
    schedule: &ScheduleType,
    last_run: Option<DateTime<Utc>>,
) -> Option<DateTime<Utc>> {
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
            if time_parts.is_empty() {
                return None;
            }
            let h = time_parts
                .get(0)
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(0);
            let m = time_parts
                .get(1)
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(0);
            let s = time_parts
                .get(2)
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(0);

            let local_now = chrono::Local::now();
            let naive_time = NaiveTime::from_hms_opt(h, m, s)?;
            let naive_date = local_now.date_naive();

            let mut next_local = naive_date
                .and_time(naive_time)
                .and_local_timezone(chrono::Local)
                .single()?;
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

struct DueTask {
    id: String,
    action: TaskAction,
    priority: u8,
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

    pub fn update_task(&self, mut updated_task: ScheduledTask) -> Result<(), String> {
        let mut tasks = self.tasks.lock();
        if !tasks.contains_key(&updated_task.id) {
            return Err(format!("任务不存在: {}", updated_task.id));
        }

        // 如果任务已启用，基于上次运行时间计算下一次运行时间
        if updated_task.enabled {
            updated_task.next_run =
                calculate_next_run(&updated_task.schedule, updated_task.last_run);
        } else {
            updated_task.next_run = None;
        }

        tracing::info!(task_id = %updated_task.id, name = %updated_task.name, "定时任务已更新");
        tasks.insert(updated_task.id.clone(), updated_task);
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

/// 后台智能任务调度引擎主循环
pub fn start_scheduler_loop(app_handle: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        tracing::info!("AutoController 后台定时任务调度引擎已启动");
        let running_tasks = Arc::new(Mutex::new(HashSet::<String>::new()));
        let dispatch_permits = Arc::new(Semaphore::new(MAX_CONCURRENT_DISPATCHES));
        let mut interval =
            tokio::time::interval(std::time::Duration::from_millis(SCHEDULER_TICK_MS));
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            interval.tick().await;

            let queue = match app_handle.try_state::<TaskQueue>() {
                Some(q) => q,
                None => continue,
            };

            let now = Utc::now();
            let tasks_to_run = collect_due_tasks(&queue, now, &running_tasks);

            if tasks_to_run.is_empty() {
                continue;
            }

            for task in tasks_to_run {
                advance_task(&queue, &task.id, now);
                queue.persist();
                dispatch_task(
                    app_handle.clone(),
                    task,
                    running_tasks.clone(),
                    dispatch_permits.clone(),
                );
            }
        }
    });
}

fn collect_due_tasks(
    queue: &TaskQueue,
    now: DateTime<Utc>,
    running_tasks: &Arc<Mutex<HashSet<String>>>,
) -> Vec<DueTask> {
    let tasks = queue.tasks.lock();
    let mut running = running_tasks.lock();
    let mut due = Vec::new();

    for task in tasks.values() {
        let should_run = task.enabled && task.next_run.is_some_and(|next| next <= now);
        if should_run && !running.contains(&task.id) {
            running.insert(task.id.clone());
            due.push(DueTask {
                id: task.id.clone(),
                action: task.action.clone(),
                priority: task.priority,
            });
        }
    }

    due.sort_by(|a, b| b.priority.cmp(&a.priority));
    due
}

fn advance_task(queue: &TaskQueue, task_id: &str, now: DateTime<Utc>) {
    let mut tasks = queue.tasks.lock();
    if let Some(task) = tasks.get_mut(task_id) {
        task.last_run = Some(now);
        if let ScheduleType::Once(_) = task.schedule {
            task.enabled = false;
            task.next_run = None;
        } else {
            task.next_run = calculate_next_run(&task.schedule, Some(now));
        }
    }
}

fn dispatch_task(
    app_handle: tauri::AppHandle,
    task: DueTask,
    running_tasks: Arc<Mutex<HashSet<String>>>,
    dispatch_permits: Arc<Semaphore>,
) {
    tauri::async_runtime::spawn(async move {
        let task_id = task.id.clone();
        let permit = match dispatch_permits.acquire_owned().await {
            Ok(permit) => permit,
            Err(e) => {
                running_tasks.lock().remove(&task_id);
                tracing::error!(task_id = %task_id, error = %e, "调度并发控制器已关闭");
                return;
            }
        };

        tracing::info!(task_id = %task_id, "触发定时调度任务");
        let result = execute_task_action(app_handle, &task_id, &task.action);
        drop(permit);

        running_tasks.lock().remove(&task_id);
        if let Err(e) = result {
            tracing::error!(task_id = %task_id, error = %e, "定时调度任务执行失败");
        } else {
            tracing::info!(task_id = %task_id, "定时调度任务执行成功");
        }
    });
}

fn execute_task_action(
    app_handle: tauri::AppHandle,
    task_id: &str,
    action: &TaskAction,
) -> Result<(), String> {
    match action {
        TaskAction::PlayMacro {
            macro_id,
            speed,
            loop_count,
        } => {
            if let (Some(player), Some(recorder), Some(controller)) = (
                app_handle.try_state::<MacroPlayer>(),
                app_handle.try_state::<crate::macro_engine::MacroRecorder>(),
                app_handle.try_state::<ControllerManager>(),
            ) {
                if let Some(mac) = recorder.get_macro(macro_id) {
                    player
                        .start_playback(controller.inner().clone(), mac, *speed, *loop_count)
                        .map(|_| ())
                } else {
                    Err("指定手柄宏已不存在".to_string())
                }
            } else {
                Err("手柄模拟基础设施未就绪".to_string())
            }
        }
        TaskAction::ExecuteScript { script_id } => {
            if let Some(runtime) = app_handle.try_state::<ScriptRuntime>() {
                runtime.execute_script(script_id).map(|_| ())
            } else {
                Err("脚本自动化引擎未就绪".to_string())
            }
        }
        TaskAction::ExecuteSequence {
            steps,
            task_loop_count,
        } => {
            if let Some(runtime) = app_handle.try_state::<ScriptRuntime>() {
                runtime.execute_sequence(task_id, steps.clone(), *task_loop_count)
            } else {
                Err("脚本引擎未就绪".to_string())
            }
        }
    }
}
