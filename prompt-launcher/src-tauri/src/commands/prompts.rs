use crate::prompts::PromptEntry;
use crate::services::prompts_service::PromptsService;
use crate::state::AppState;
use std::sync::Arc;
use tauri::{AppHandle, State};

#[tauri::command]
pub fn list_prompts(state: State<Arc<AppState>>) -> Vec<PromptEntry> {
    PromptsService::list(state.inner())
}

#[tauri::command]
pub fn search_prompts(
    state: State<Arc<AppState>>,
    query: String,
    limit: usize,
    favorites_only: bool,
) -> Vec<PromptEntry> {
    PromptsService::search(state.inner(), &query, limit, favorites_only)
}

#[tauri::command]
pub fn set_prompts_dir(
    app: AppHandle,
    state: State<Arc<AppState>>,
    path: String,
) -> Result<Vec<PromptEntry>, String> {
    PromptsService::set_prompts_dir(&app, state.inner(), path)
}

#[tauri::command]
pub fn create_prompt_file(
    state: State<Arc<AppState>>,
    name: String,
) -> Result<String, String> {
    PromptsService::create_prompt_file(state.inner(), name)
}

#[tauri::command]
pub fn open_prompt_path(
    app: AppHandle,
    state: State<Arc<AppState>>,
    path: String,
) -> Result<(), String> {
    PromptsService::open_prompt_path(&app, state.inner(), path)
}

#[tauri::command]
pub fn delete_prompt_files(
    app: AppHandle,
    state: State<Arc<AppState>>,
    paths: Vec<String>,
) -> Result<Vec<PromptEntry>, String> {
    PromptsService::delete_prompt_files(&app, state.inner(), paths)
}

#[tauri::command]
pub fn update_prompt_tags(
    state: State<Arc<AppState>>,
    paths: Vec<String>,
    add: Vec<String>,
    remove: Vec<String>,
) -> Result<Vec<PromptEntry>, String> {
    PromptsService::update_prompt_tags(state.inner(), paths, add, remove)
}
