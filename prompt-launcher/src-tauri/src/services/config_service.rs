use crate::config::{save, AppConfig};
use crate::services::prompts_service::PromptsService;
use crate::services::window_service::WindowService;
use crate::state::{AppState, RecentState};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::AppHandle;

pub struct ConfigService;

impl ConfigService {
    pub fn get_config(state: &AppState) -> AppConfig {
        state.config.lock().unwrap().clone()
    }

    pub fn set_auto_paste(
        app: &AppHandle,
        state: &Arc<AppState>,
        auto_paste: bool,
    ) -> Result<(), String> {
        let mut config = state.config.lock().unwrap();
        config.auto_paste = auto_paste;
        save(app, &config)
    }

    pub fn set_append_clipboard(
        app: &AppHandle,
        state: &Arc<AppState>,
        append_clipboard: bool,
    ) -> Result<(), String> {
        let mut config = state.config.lock().unwrap();
        config.append_clipboard = append_clipboard;
        save(app, &config)
    }

    pub fn set_hotkey(
        app: &AppHandle,
        state: &Arc<AppState>,
        hotkey: String,
    ) -> Result<(), String> {
        let trimmed = hotkey.trim();
        if trimmed.is_empty() {
            return Err("快捷键不能为空".to_string());
        }
        WindowService::update_hotkey_registration(app, state, trimmed)?;
        let mut config = state.config.lock().unwrap();
        if config.hotkey == trimmed {
            return Ok(());
        }
        config.hotkey = trimmed.to_string();
        save(app, &config)
    }

    pub fn set_auto_start(
        app: &AppHandle,
        state: &Arc<AppState>,
        auto_start: bool,
    ) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            let exe_path = std::env::current_exe()
                .map_err(|e| format!("resolve exe path failed: {e}"))?;
            crate::autostart::set_auto_start(auto_start, &exe_path)?;
        }

        let mut config = state.config.lock().unwrap();
        config.auto_start = auto_start;
        save(app, &config)
    }

    pub fn toggle_favorite(
        app: &AppHandle,
        state: &Arc<AppState>,
        id: String,
    ) -> Result<Vec<String>, String> {
        let mut config = state.config.lock().unwrap();
        if let Some(pos) = config.favorites.iter().position(|item| item == &id) {
            config.favorites.remove(pos);
        } else {
            config.favorites.push(id);
        }
        save(app, &config)?;
        Ok(config.favorites.clone())
    }

    pub fn push_recent(
        app: &AppHandle,
        state: &Arc<AppState>,
        id: String,
    ) -> Result<RecentState, String> {
        let mut config = state.config.lock().unwrap();
        config.recent_ids.retain(|item| item != &id);
        config.recent_ids.insert(0, id);
        if config.recent_ids.len() > 20 {
            config.recent_ids.truncate(20);
        }
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("time error: {e}"))?
            .as_millis() as i64;
        let current_id = config.recent_ids.first().cloned().unwrap_or_default();
        if !current_id.is_empty() {
            config.recent_meta.insert(current_id, now);
        }
        let keep: HashSet<String> = config.recent_ids.iter().cloned().collect();
        config.recent_meta.retain(|key, _| keep.contains(key));
        save(app, &config)?;
        Ok(RecentState {
            recent_ids: config.recent_ids.clone(),
            recent_meta: config.recent_meta.clone(),
        })
    }

    pub fn set_recent_enabled(
        app: &AppHandle,
        state: &Arc<AppState>,
        recent_enabled: bool,
    ) -> Result<(), String> {
        let mut config = state.config.lock().unwrap();
        config.recent_enabled = recent_enabled;
        save(app, &config)
    }

    pub fn set_top_tags_scope(
        app: &AppHandle,
        state: &Arc<AppState>,
        use_results: bool,
    ) -> Result<(), String> {
        let mut config = state.config.lock().unwrap();
        config.top_tags_use_results = use_results;
        save(app, &config)
    }

    pub fn set_top_tags_limit(
        app: &AppHandle,
        state: &Arc<AppState>,
        limit: u32,
    ) -> Result<(), String> {
        let mut config = state.config.lock().unwrap();
        let value = limit.clamp(1, 20);
        config.top_tags_limit = value;
        save(app, &config)
    }

    pub fn set_preview_chars(
        app: &AppHandle,
        state: &Arc<AppState>,
        preview_chars: u32,
    ) -> Result<(), String> {
        let value = PromptsService::clamp_preview_chars(preview_chars);
        {
            let mut config = state.config.lock().unwrap();
            config.preview_chars = value;
            save(app, &config)?;
        }
        PromptsService::apply_preview_chars(state, value);
        Ok(())
    }

    pub fn set_show_shortcuts_hint(
        app: &AppHandle,
        state: &Arc<AppState>,
        show_shortcuts_hint: bool,
    ) -> Result<(), String> {
        let mut config = state.config.lock().unwrap();
        config.show_shortcuts_hint = show_shortcuts_hint;
        save(app, &config)
    }

    pub fn clear_recent(
        app: &AppHandle,
        state: &Arc<AppState>,
    ) -> Result<RecentState, String> {
        let mut config = state.config.lock().unwrap();
        config.recent_ids.clear();
        config.recent_meta.clear();
        save(app, &config)?;
        Ok(RecentState {
            recent_ids: config.recent_ids.clone(),
            recent_meta: config.recent_meta.clone(),
        })
    }
}
