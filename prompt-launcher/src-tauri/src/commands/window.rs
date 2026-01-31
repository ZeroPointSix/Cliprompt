use crate::services::window_service::WindowService;
use crate::state::AppState;
use std::sync::Arc;
use tauri::{AppHandle, State};

#[tauri::command]
pub fn frontend_ready(app: AppHandle, state: State<Arc<AppState>>) -> Result<(), String> {
    WindowService::frontend_ready(&app, state.inner())
}

#[tauri::command]
pub fn capture_active_window(state: State<Arc<AppState>>) -> Result<(), String> {
    WindowService::capture_active_window(state.inner())
}

#[tauri::command]
pub fn focus_last_window(
    state: State<Arc<AppState>>,
    auto_paste: bool,
) -> Result<(), String> {
    WindowService::focus_last_window(state.inner(), auto_paste)
}
