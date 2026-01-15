mod config;
mod prompts;
mod win;

use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, State,
};

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
    store_active_window(state.inner())
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

fn store_active_window(state: &Arc<AppState>) -> Result<(), String> {
    if let Some(hwnd) = win::capture_foreground_window() {
        *state.last_active_hwnd.lock().unwrap() = Some(hwnd);
        Ok(())
    } else {
        Err("no active window detected".to_string())
    }
}

fn main_window(app: &AppHandle) -> Result<tauri::WebviewWindow, String> {
    app.get_webview_window("main")
        .ok_or_else(|| "main window not found".to_string())
}

fn show_main_window(app: &AppHandle) -> Result<(), String> {
    let window = main_window(app)?;
    window
        .show()
        .map_err(|e| format!("show window failed: {e}"))?;
    window
        .set_focus()
        .map_err(|e| format!("focus window failed: {e}"))?;
    Ok(())
}

fn hide_main_window(app: &AppHandle) -> Result<(), String> {
    let window = main_window(app)?;
    window
        .hide()
        .map_err(|e| format!("hide window failed: {e}"))?;
    Ok(())
}

fn toggle_main_window(app: &AppHandle) -> Result<(), String> {
    let window = main_window(app)?;
    let visible = window
        .is_visible()
        .map_err(|e| format!("query window failed: {e}"))?;
    if visible {
        hide_main_window(app)
    } else {
        show_main_window(app)
    }
}

fn init_tray(app: &tauri::App) -> Result<(), String> {
    let show = MenuItem::with_id(app, "show", "Show", true, None)
        .map_err(|e| format!("menu item failed: {e}"))?;
    let hide = MenuItem::with_id(app, "hide", "Hide", true, None)
        .map_err(|e| format!("menu item failed: {e}"))?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None)
        .map_err(|e| format!("menu item failed: {e}"))?;
    let menu = Menu::with_items(app, &[&show, &hide, &quit])
        .map_err(|e| format!("menu build failed: {e}"))?;
    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| "missing default window icon".to_string())?;

    TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => {
                let state = app.state::<Arc<AppState>>();
                let _ = store_active_window(state.inner());
                let _ = show_main_window(app);
            }
            "hide" => {
                let _ = hide_main_window(app);
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if matches!(event, TrayIconEvent::Click { .. }) {
                let app = tray.app_handle();
                let state = app.state::<Arc<AppState>>();
                let _ = store_active_window(state.inner());
                let _ = toggle_main_window(&app);
            }
        })
        .build(app)
        .map_err(|e| format!("tray init failed: {e}"))?;

    Ok(())
}

fn seed_prompts_if_empty(dir: &Path) -> Result<(), String> {
    if !dir.exists() {
        return Ok(());
    }
    if !index_prompts(dir).is_empty() {
        return Ok(());
    }

    let quickstart = dir.join("Quickstart #welcome.md");
    if !quickstart.exists() {
        let content = "\
Prompt Launcher Quickstart

- Use the global hotkey to open the launcher.
- Type to search, including #tags.
- Enter pastes. Right click opens the file.
";
        fs::write(&quickstart, content).map_err(|e| format!("seed quickstart failed: {e}"))?;
    }

    let examples_dir = dir.join("Examples");
    fs::create_dir_all(&examples_dir).map_err(|e| format!("seed dir failed: {e}"))?;
    let email = examples_dir.join("Email reply #email.txt");
    if !email.exists() {
        let content = "\
Reply in a friendly, concise tone.
Summarize the request, confirm next steps, and ask for missing details.
";
        fs::write(&email, content).map_err(|e| format!("seed email failed: {e}"))?;
    }

    Ok(())
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
            seed_prompts_if_empty(&dir)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

            let state = Arc::new(AppState::new(config));
            let prompts = index_prompts(&dir);
            {
                let mut lock = state.prompts.write().unwrap();
                *lock = prompts;
            }

            app.manage(state.clone());
            init_tray(app)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
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
