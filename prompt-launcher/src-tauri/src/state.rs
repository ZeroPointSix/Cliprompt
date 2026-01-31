use crate::lifecycle::LauncherGate;
use crate::prompts::PromptEntry;
use notify::RecommendedWatcher;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Mutex, RwLock};

pub struct AppState {
    pub(crate) prompts: RwLock<Vec<PromptEntry>>,
    pub(crate) config: Mutex<crate::config::AppConfig>,
    pub(crate) watcher: Mutex<Option<RecommendedWatcher>>,
    pub(crate) last_active_hwnd: Mutex<Option<isize>>,
    pub(crate) pending_paths: Mutex<HashMap<String, u128>>,
    pub(crate) registered_hotkey: Mutex<Option<String>>,
    pub(crate) launcher_gate: Mutex<LauncherGate>,
}

#[derive(Serialize)]
pub struct RecentState {
    pub recent_ids: Vec<String>,
    pub recent_meta: HashMap<String, i64>,
}

impl AppState {
    pub fn new(config: crate::config::AppConfig) -> Self {
        Self {
            prompts: RwLock::new(Vec::new()),
            config: Mutex::new(config),
            watcher: Mutex::new(None),
            last_active_hwnd: Mutex::new(None),
            pending_paths: Mutex::new(HashMap::new()),
            registered_hotkey: Mutex::new(None),
            launcher_gate: Mutex::new(LauncherGate::new()),
        }
    }
}
