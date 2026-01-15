mod config;
mod prompts;
mod win;

use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Manager, State};

use config::{load_or_init, save, AppConfig};
use prompts::{index_prompts, PromptEntry};

struct AppState {
    prompts: RwLock<Vec<PromptEntry>>,
    config: Mutex<AppConfig>,
    watcher: Mutex<Option<RecommendedWatcher>>,
    last_active_hwnd: Mutex<Option<isize>>,
}

impl AppState {
    fn new(config: AppConfig) -> Self {
        Self {
            prompts: RwLock::new(Vec::new()),
            config: Mutex::new(config),
            watcher: Mutex::new(None),
            last_active_hwnd: Mutex::new(None),
        }
    }
}

#[tauri::command]
fn get_config(state: State<Arc<AppState>>) -> AppConfig {
    state.config.lock().unwrap().clone()
}

#[tauri::command]
fn list_prompts(state: State<Arc<AppState>>) -> Vec<PromptEntry> {
    state.prompts.read().unwrap().clone()
}

#[tauri::command]
fn set_prompts_dir(
    app: AppHandle,
    state: State<Arc<AppState>>,
    path: String,
) -> Result<Vec<PromptEntry>, String> {
    let dir = PathBuf::from(&path);
    fs::create_dir_all(&dir).map_err(|e| format!("create prompts dir failed: {e}"))?;

    {
        let mut config = state.config.lock().unwrap();
        config.prompts_dir = dir.to_string_lossy().to_string();
        save(&app, &config)?;
    }

    let prompts = refresh_prompts(state.inner(), &dir);
    start_watcher(app, state.inner().clone(), dir)?;
    Ok(prompts)
}

#[tauri::command]
fn set_auto_paste(
    app: AppHandle,
    state: State<Arc<AppState>>,
    auto_paste: bool,
) -> Result<(), String> {
    let mut config = state.config.lock().unwrap();
    config.auto_paste = auto_paste;
    save(&app, &config)
}

#[tauri::command]
fn set_hotkey(
    app: AppHandle,
    state: State<Arc<AppState>>,
    hotkey: String,
) -> Result<(), String> {
    let mut config = state.config.lock().unwrap();
    config.hotkey = hotkey;
    save(&app, &config)
}

#[tauri::command]
fn capture_active_window(state: State<Arc<AppState>>) -> Result<(), String> {
    if let Some(hwnd) = win::capture_foreground_window() {
        *state.last_active_hwnd.lock().unwrap() = Some(hwnd);
        Ok(())
    } else {
        Err("no active window detected".to_string())
    }
}

#[tauri::command]
fn focus_last_window(
    state: State<Arc<AppState>>,
    auto_paste: bool,
) -> Result<(), String> {
    let hwnd = *state.last_active_hwnd.lock().unwrap();
    if let Some(hwnd) = hwnd {
        win::focus_window(hwnd)?;
        if auto_paste {
            thread::sleep(Duration::from_millis(30));
            win::send_ctrl_v()?;
        }
    }
    Ok(())
}

fn refresh_prompts(state: &Arc<AppState>, dir: &Path) -> Vec<PromptEntry> {
    let prompts = index_prompts(dir);
    let mut lock = state.prompts.write().unwrap();
    *lock = prompts.clone();
    prompts
}

fn start_watcher(
    app: AppHandle,
    state: Arc<AppState>,
    dir: PathBuf,
) -> Result<(), String> {
    let index_dir = dir.clone();
    let app_handle = app.clone();
    let state_handle = state.clone();
    let mut watcher = notify::recommended_watcher(move |res| {
        if res.is_err() {
            return;
        }
        let prompts = index_prompts(&index_dir);
        {
            let mut lock = state_handle.prompts.write().unwrap();
            *lock = prompts.clone();
        }
        let _ = app_handle.emit("prompts-updated", prompts);
    })
    .map_err(|e| format!("watcher init failed: {e}"))?;

    watcher
        .watch(&dir, RecursiveMode::Recursive)
        .map_err(|e| format!("watcher start failed: {e}"))?;

    *state.watcher.lock().unwrap() = Some(watcher);
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_global_shortcut::init())
        .setup(|app| {
            let handle = app.handle();
            let config = load_or_init(handle)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

            let dir = PathBuf::from(&config.prompts_dir);
            fs::create_dir_all(&dir)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

            let state = Arc::new(AppState::new(config));
            let prompts = index_prompts(&dir);
            {
                let mut lock = state.prompts.write().unwrap();
                *lock = prompts;
            }

            app.manage(state.clone());
            start_watcher(handle.clone(), state, dir)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            list_prompts,
            set_prompts_dir,
            set_auto_paste,
            set_hotkey,
            capture_active_window,
            focus_last_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
