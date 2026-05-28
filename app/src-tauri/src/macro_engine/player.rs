use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

use crate::controller::{ControllerManager, ThumbAxis, TriggerSide};
use super::types::{Macro, MacroEventType};

#[derive(Clone)]
#[allow(dead_code)]
pub struct PlaybackState {
    pub id: String,
    pub macro_id: String,
    pub speed: f32,
    pub loop_count: u32,
    pub current_loop: u32,
    pub current_event_index: usize,
    pub paused: bool,
}

pub struct MacroPlayer {
    playbacks: Arc<Mutex<HashMap<String, PlaybackState>>>,
}

impl MacroPlayer {
    pub fn new() -> Self {
        Self {
            playbacks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn start_playback(
        &self,
        controller: ControllerManager,
        mac: Macro,
        speed: f32,
        loop_count: u32,
    ) -> Result<String, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let playback = PlaybackState {
            id: id.clone(),
            macro_id: mac.id.clone(),
            speed: speed.clamp(0.5, 2.0),
            loop_count,
            current_loop: 0,
            current_event_index: 0,
            paused: false,
        };

        {
            let mut playbacks = self.playbacks.lock();
            playbacks.insert(id.clone(), playback);
        }

        let playbacks_clone = self.playbacks.clone();
        let id_clone = id.clone();

        std::thread::spawn(move || {
            let speed_factor = speed.clamp(0.5, 2.0);
            let events = mac.events;
            if events.is_empty() {
                let mut playbacks = playbacks_clone.lock();
                playbacks.remove(&id_clone);
                return;
            }

            let mut current_loop = 0;
            loop {
                // 如果指定了循环次数，并且达到上限，则终止
                if loop_count > 0 && current_loop >= loop_count {
                    break;
                }

                {
                    let mut playbacks = playbacks_clone.lock();
                    if let Some(state) = playbacks.get_mut(&id_clone) {
                        state.current_loop = current_loop;
                        state.current_event_index = 0;
                    } else {
                        break; // 被手动停止 (Stopped)
                    }
                }

                let loop_start_time = std::time::Instant::now();
                let mut next_event_idx = 0;
                let mut pause_offset = std::time::Duration::ZERO;

                while next_event_idx < events.len() {
                    // 检测暂停状态
                    let mut is_paused = {
                        let playbacks = playbacks_clone.lock();
                        if let Some(state) = playbacks.get(&id_clone) {
                            state.paused
                        } else {
                            break; // 被手动停止 (Stopped)
                        }
                    };

                    if is_paused {
                        let pause_start = std::time::Instant::now();
                        while is_paused {
                            std::thread::sleep(std::time::Duration::from_millis(10));
                            {
                                let playbacks = playbacks_clone.lock();
                                if let Some(state) = playbacks.get(&id_clone) {
                                    is_paused = state.paused;
                                } else {
                                    break; // 被手动停止 (Stopped)
                                }
                            }
                        }
                        pause_offset += pause_start.elapsed();
                    }

                    // 再次检查在此期间是否被手动删除/停止
                    {
                        let playbacks = playbacks_clone.lock();
                        if !playbacks.contains_key(&id_clone) {
                            break;
                        }
                    }

                    let event = &events[next_event_idx];
                    let elapsed = loop_start_time.elapsed().checked_sub(pause_offset).unwrap_or(std::time::Duration::ZERO);
                    let target_time = std::time::Duration::from_millis((event.timestamp_ms as f32 / speed_factor) as u64);

                    if elapsed < target_time {
                        let to_sleep = target_time - elapsed;
                        // 细颗粒休眠（5ms），以便后台能极快响应暂停/停止
                        if to_sleep > std::time::Duration::from_millis(5) {
                            std::thread::sleep(std::time::Duration::from_millis(5));
                            continue;
                        } else {
                            std::thread::sleep(to_sleep);
                        }
                    }

                    // 派发并执行宏事件
                    match &event.event_type {
                        MacroEventType::ButtonPress(btn) => {
                            let _ = controller.set_button(&event.device_id, *btn, true);
                        }
                        MacroEventType::ButtonRelease(btn) => {
                            let _ = controller.set_button(&event.device_id, *btn, false);
                        }
                        MacroEventType::ThumbMove(axis_str, x, y) => {
                            if axis_str == "left" {
                                let _ = controller.set_thumb(&event.device_id, ThumbAxis::LeftX, *x);
                                let _ = controller.set_thumb(&event.device_id, ThumbAxis::LeftY, *y);
                            } else if axis_str == "right" {
                                let _ = controller.set_thumb(&event.device_id, ThumbAxis::RightX, *x);
                                let _ = controller.set_thumb(&event.device_id, ThumbAxis::RightY, *y);
                            }
                        }
                        MacroEventType::TriggerMove(side_str, value) => {
                            if side_str == "left" || side_str == "l" {
                                let _ = controller.set_trigger(&event.device_id, TriggerSide::Left, *value);
                            } else if side_str == "right" || side_str == "r" {
                                let _ = controller.set_trigger(&event.device_id, TriggerSide::Right, *value);
                            }
                        }
                    }

                    next_event_idx += 1;
                    {
                        let mut playbacks = playbacks_clone.lock();
                        if let Some(state) = playbacks.get_mut(&id_clone) {
                            state.current_event_index = next_event_idx;
                        }
                    }
                }

                // 检查是否结束
                {
                    let playbacks = playbacks_clone.lock();
                    if !playbacks.contains_key(&id_clone) {
                        break;
                    }
                }

                current_loop += 1;
            }

            // 清理状态
            {
                let mut playbacks = playbacks_clone.lock();
                playbacks.remove(&id_clone);
            }
            tracing::info!(playback_id = %id_clone, "回放线程执行结束");
        });

        Ok(id)
    }

    pub fn pause(&self, playback_id: &str) -> Result<(), String> {
        let mut playbacks = self.playbacks.lock();
        if let Some(state) = playbacks.get_mut(playback_id) {
            state.paused = true;
            Ok(())
        } else {
            Err(format!("回放不存在: {}", playback_id))
        }
    }

    pub fn resume(&self, playback_id: &str) -> Result<(), String> {
        let mut playbacks = self.playbacks.lock();
        if let Some(state) = playbacks.get_mut(playback_id) {
            state.paused = false;
            Ok(())
        } else {
            Err(format!("回放不存在: {}", playback_id))
        }
    }

    pub fn stop(&self, playback_id: &str) -> Result<(), String> {
        let mut playbacks = self.playbacks.lock();
        if playbacks.remove(playback_id).is_some() {
            tracing::info!(playback_id = %playback_id, "回放已停止");
            Ok(())
        } else {
            Err(format!("回放不存在: {}", playback_id))
        }
    }

    pub fn get_playback(&self, playback_id: &str) -> Option<PlaybackState> {
        let playbacks = self.playbacks.lock();
        playbacks.get(playback_id).cloned()
    }
}
