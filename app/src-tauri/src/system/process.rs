use std::sync::Arc;
use parking_lot::Mutex;

#[allow(dead_code)]
pub struct ProcessMonitor {
    watching: Arc<Mutex<Vec<String>>>,
}

impl ProcessMonitor {
    pub fn new() -> Self {
        Self {
            watching: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn add_process(&self, process_name: &str) {
        let mut watching = self.watching.lock();
        if !watching.contains(&process_name.to_lowercase()) {
            watching.push(process_name.to_lowercase());
        }
    }

    pub fn remove_process(&self, process_name: &str) {
        let mut watching = self.watching.lock();
        watching.retain(|p| p != &process_name.to_lowercase());
    }

    pub fn is_game_running(&self, process_name: &str) -> bool {
        #[cfg(target_os = "windows")]
        {
            use std::process::Command;
            let output = Command::new("tasklist")
                .args(["/FI", &format!("IMAGENAME eq {}", process_name), "/NH"])
                .output();
            if let Ok(output) = output {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.to_lowercase().contains(&process_name.to_lowercase())
            } else {
                false
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = process_name;
            false
        }
    }

    pub fn list_watching(&self) -> Vec<String> {
        self.watching.lock().clone()
    }
}
