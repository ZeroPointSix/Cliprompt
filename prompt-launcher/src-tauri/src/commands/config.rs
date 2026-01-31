use crate::config::AppConfig;
use crate::services::config_service::ConfigService;
use crate::state::{AppState, RecentState};
use std::sync::Arc;
use tauri::{AppHandle, State};

#[tauri::command]
pub fn get_config(state: State<Arc<AppState>>) -> AppConfig {
    ConfigService::get_config(state.inner())
}

#[tauri::command]
pub fn set_auto_paste(
    app: AppHandle,
    state: State<Arc<AppState>>,
    auto_paste: bool,
) -> Result<(), String> {
    ConfigService::set_auto_paste(&app, state.inner(), auto_paste)
}

#[tauri::command]
pub fn set_append_clipboard(
    app: AppHandle,
    state: State<Arc<AppState>>,
    append_clipboard: bool,
) -> Result<(), String> {
    ConfigService::set_append_clipboard(&app, state.inner(), append_clipboard)
}

#[tauri::command]
pub fn set_hotkey(
    app: AppHandle,
    state: State<Arc<AppState>>,
    hotkey: String,
) -> Result<(), String> {
    ConfigService::set_hotkey(&app, state.inner(), hotkey)
}

#[tauri::command]
pub fn set_auto_start(
    app: AppHandle,
    state: State<Arc<AppState>>,
    auto_start: bool,
) -> Result<(), String> {
    ConfigService::set_auto_start(&app, state.inner(), auto_start)
}

#[tauri::command]
pub fn toggle_favorite(
    app: AppHandle,
    state: State<Arc<AppState>>,
    id: String,
) -> Result<Vec<String>, String> {
    ConfigService::toggle_favorite(&app, state.inner(), id)
}

#[tauri::command]
pub fn push_recent(
    app: AppHandle,
    state: State<Arc<AppState>>,
    id: String,
) -> Result<RecentState, String> {
    ConfigService::push_recent(&app, state.inner(), id)
}

#[tauri::command]
pub fn set_recent_enabled(
    app: AppHandle,
    state: State<Arc<AppState>>,
    recent_enabled: bool,
) -> Result<(), String> {
    ConfigService::set_recent_enabled(&app, state.inner(), recent_enabled)
}

#[tauri::command]
pub fn set_top_tags_scope(
    app: AppHandle,
    state: State<Arc<AppState>>,
    use_results: bool,
) -> Result<(), String> {
    ConfigService::set_top_tags_scope(&app, state.inner(), use_results)
}

#[tauri::command]
pub fn set_top_tags_limit(
    app: AppHandle,
    state: State<Arc<AppState>>,
    limit: u32,
) -> Result<(), String> {
    ConfigService::set_top_tags_limit(&app, state.inner(), limit)
}

#[tauri::command]
pub fn set_preview_chars(
    app: AppHandle,
    state: State<Arc<AppState>>,
    preview_chars: u32,
) -> Result<(), String> {
    ConfigService::set_preview_chars(&app, state.inner(), preview_chars)
}

#[tauri::command]
pub fn set_show_shortcuts_hint(
    app: AppHandle,
    state: State<Arc<AppState>>,
    show_shortcuts_hint: bool,
) -> Result<(), String> {
    ConfigService::set_show_shortcuts_hint(&app, state.inner(), show_shortcuts_hint)
}

#[tauri::command]
pub fn clear_recent(
    app: AppHandle,
    state: State<Arc<AppState>>,
) -> Result<RecentState, String> {
    ConfigService::clear_recent(&app, state.inner())
}
