mod commands;
mod config;
mod constants;
mod domain;
mod infrastructure;
mod lifecycle;
mod prompts;
mod services;
mod state;
mod tags_meta;
mod usecase;

#[cfg(target_os = "windows")]
mod autostart;
#[cfg(target_os = "windows")]
mod win;

use crate::commands::*;
use crate::config::load_or_init;
use crate::prompts::index_prompts;
use crate::services::prompts_service::PromptsService;
use crate::services::window_service::WindowService;
use crate::state::AppState;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let handle = app.handle();
            let config = load_or_init(handle)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            let preview_chars = PromptsService::clamp_preview_chars(config.preview_chars) as usize;

            let dir = PathBuf::from(&config.prompts_dir);
            fs::create_dir_all(&dir)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            PromptsService::seed_prompts_if_empty(&dir, preview_chars)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

            #[cfg(target_os = "windows")]
            {
                let _ = std::env::current_exe()
                    .map_err(|e| format!("resolve exe path failed: {e}"))
                    .and_then(|exe| autostart::set_auto_start(config.auto_start, &exe));
            }

            let hotkey = config.hotkey.clone();
            let state = Arc::new(AppState::new(config));
            let prompts = index_prompts(&dir, preview_chars);
            {
                let mut lock = state.prompts.write().unwrap();
                *lock = prompts;
            }

            app.manage(state.clone());
            if let Err(error) = WindowService::update_hotkey_registration(&handle, &state, &hotkey)
            {
                eprintln!("[hotkey] register failed: {error}");
            }
            WindowService::init_tray(app)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            PromptsService::start_watcher(handle.clone(), state.clone(), dir)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            if cfg!(debug_assertions) {
                let _ = WindowService::request_show(&handle, &state);
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            frontend_ready,
            list_prompts,
            search_prompts,
            set_prompts_dir,
            create_prompt_file,
            open_prompt_path,
            delete_prompt_files,
            set_auto_paste,
            set_append_clipboard,
            set_hotkey,
            set_auto_start,
            toggle_favorite,
            push_recent,
            set_recent_enabled,
            update_prompt_tags,
            set_top_tags_scope,
            set_top_tags_limit,
            set_preview_chars,
            set_show_shortcuts_hint,
            clear_recent,
            capture_active_window,
            focus_last_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
