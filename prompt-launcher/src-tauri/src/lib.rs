mod autostart;
mod config;
mod prompts;
mod tags_meta;
mod win;

use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use std::time::Duration;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, State,
};
use tauri::Emitter;
use tauri_plugin_opener::OpenerExt;

use config::{load_or_init, save, AppConfig};
use prompts::{index_prompts, search_prompts as search_prompts_impl, PromptEntry, normalize_tag};
use tags_meta::{load_tags_meta, path_to_key, save_tags_meta, touch_updated_at};

struct AppState {
    prompts: RwLock<Vec<PromptEntry>>,
    config: Mutex<AppConfig>,
    watcher: Mutex<Option<RecommendedWatcher>>,
    last_active_hwnd: Mutex<Option<isize>>,
    pending_paths: Mutex<HashSet<String>>,
}

#[derive(Serialize)]
struct RecentState {
    recent_ids: Vec<String>,
    recent_meta: std::collections::HashMap<String, i64>,
}

impl AppState {
    fn new(config: AppConfig) -> Self {
        Self {
            prompts: RwLock::new(Vec::new()),
            config: Mutex::new(config),
            watcher: Mutex::new(None),
            last_active_hwnd: Mutex::new(None),
            pending_paths: Mutex::new(HashSet::new()),
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

fn resolve_prompts_root(state: &AppState) -> Result<PathBuf, String> {
    let dir = state.config.lock().unwrap().prompts_dir.clone();
    if dir.trim().is_empty() {
        return Err("提示词目录未配置".to_string());
    }
    let root = PathBuf::from(dir);
    root.canonicalize()
        .map_err(|e| format!("解析提示词目录失败: {e}"))
}

fn resolve_prompt_path(root: &Path, path: &Path) -> Result<PathBuf, String> {
    let target = path
        .canonicalize()
        .map_err(|e| format!("解析路径失败: {e}"))?;
    if !target.starts_with(root) {
        return Err("路径不在提示词目录中".to_string());
    }
    Ok(target)
}

#[tauri::command]
fn search_prompts(
    state: State<Arc<AppState>>,
    query: String,
    limit: usize,
    favorites_only: bool,
) -> Vec<PromptEntry> {
    let prompts = state.prompts.read().unwrap();
    if favorites_only {
        let favorites = state.config.lock().unwrap().favorites.clone();
        let favorites: HashSet<String> = favorites.into_iter().collect();
        let filtered: Vec<PromptEntry> = prompts
            .iter()
            .filter(|prompt| favorites.contains(&prompt.id))
            .cloned()
            .collect();
        return search_prompts_impl(&filtered, &query, limit);
    }
    search_prompts_impl(&prompts, &query, limit)
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

    {
        let mut pending = state.pending_paths.lock().unwrap();
        pending.clear();
    }

    let prompts = refresh_prompts(state.inner(), &dir);
    start_watcher(app, state.inner().clone(), dir)?;
    Ok(prompts)
}

#[tauri::command]
fn create_prompt_file(
    state: State<Arc<AppState>>,
    name: String,
) -> Result<String, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("文件名不能为空".to_string());
    }
    if !is_valid_filename(trimmed) {
        return Err("文件名包含非法字符".to_string());
    }
    let file_name = if trimmed.to_ascii_lowercase().ends_with(".txt") {
        trimmed.to_string()
    } else {
        format!("{trimmed}.txt")
    };
    if !is_valid_filename(&file_name) {
        return Err("文件名包含非法字符".to_string());
    }
    let dir = {
        let config = state.config.lock().unwrap();
        config.prompts_dir.clone()
    };
    if dir.trim().is_empty() {
        return Err("提示词目录未配置".to_string());
    }
    fs::create_dir_all(&dir).map_err(|e| format!("create prompts dir failed: {e}"))?;
    let path = PathBuf::from(dir).join(&file_name);
    if path.exists() {
        return Err("文件已存在，无法创建".to_string());
    }
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|e| format!("create file failed: {e}"))?;
    let path_string = path.to_string_lossy().to_string();
    state
        .pending_paths
        .lock()
        .unwrap()
        .insert(path_string.clone());
    Ok(path_string)
}

#[tauri::command]
fn open_prompt_path(
    app: AppHandle,
    state: State<Arc<AppState>>,
    path: String,
) -> Result<(), String> {
    let root = resolve_prompts_root(state.inner())?;
    let target = resolve_prompt_path(&root, Path::new(&path))?;
    let target_str = target.to_string_lossy().to_string();
    #[cfg(target_os = "windows")]
    {
        let mut errors = Vec::new();
        let candidates = ["code", "code.cmd", "notepad", "notepad.exe"];
        for editor in candidates {
            match app.opener().open_path(target_str.clone(), Some(editor)) {
                Ok(_) => return Ok(()),
                Err(err) => errors.push(format!("{editor}: {err}")),
            }
        }
        if let Err(err) = app.opener().open_path(target_str.clone(), None::<&str>) {
            errors.push(format!("default: {err}"));
            return Err(format!("打开失败：{}", errors.join(" | ")));
        }
        return Ok(());
    }

    #[allow(unreachable_code)]
    {
        app.opener()
            .open_path(target_str.clone(), None::<&str>)
            .map_err(|error| format!("打开失败：{error}"))
    }
}

#[tauri::command]
fn update_prompt_tags(
    state: State<Arc<AppState>>,
    paths: Vec<String>,
    add: Vec<String>,
    remove: Vec<String>,
) -> Result<Vec<PromptEntry>, String> {
    if paths.is_empty() {
        return Err("未选择任何提示词".to_string());
    }
    let add_tags = normalize_input_tags(add)?;
    let remove_tags = normalize_input_tags(remove)?;
    if add_tags.is_empty() && remove_tags.is_empty() {
        return Err("标签不能为空".to_string());
    }
    let dir = {
        let config = state.config.lock().unwrap();
        config.prompts_dir.clone()
    };
    if dir.trim().is_empty() {
        return Err("提示词目录未配置".to_string());
    }
    let root = PathBuf::from(&dir);
    let mut meta = load_tags_meta(&root)?;
    let prompt_map = build_prompt_tag_map(state.inner());

    for raw_path in paths {
        let path = PathBuf::from(&raw_path);
        let key = path_to_key(&root, &path);
        let base_tags = if let Some(existing) = meta.tags_by_path.get(&key) {
            existing.clone()
        } else {
            prompt_map.get(&raw_path).cloned().unwrap_or_default()
        };
        let mut next = HashSet::new();
        for tag in base_tags {
            if let Some(normalized) = normalize_tag(&tag) {
                next.insert(normalized);
            }
        }
        for tag in &add_tags {
            next.insert(tag.clone());
        }
        for tag in &remove_tags {
            next.remove(tag);
        }
        let mut next_vec: Vec<String> = next.into_iter().collect();
        next_vec.sort();
        meta.tags_by_path.insert(key, next_vec);
    }

    touch_updated_at(&mut meta);
    save_tags_meta(&root, &meta)?;
    Ok(refresh_prompts(state.inner(), &root))
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
fn set_auto_start(
    app: AppHandle,
    state: State<Arc<AppState>>,
    auto_start: bool,
) -> Result<(), String> {
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("resolve exe path failed: {e}"))?;

    autostart::set_auto_start(auto_start, &exe_path)?;

    let mut config = state.config.lock().unwrap();
    config.auto_start = auto_start;
    save(&app, &config)
}

#[tauri::command]
fn toggle_favorite(
    app: AppHandle,
    state: State<Arc<AppState>>,
    id: String,
) -> Result<Vec<String>, String> {
    let mut config = state.config.lock().unwrap();
    if let Some(pos) = config.favorites.iter().position(|item| item == &id) {
        config.favorites.remove(pos);
    } else {
        config.favorites.push(id);
    }
    save(&app, &config)?;
    Ok(config.favorites.clone())
}

#[tauri::command]
fn push_recent(
    app: AppHandle,
    state: State<Arc<AppState>>,
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
    save(&app, &config)?;
    Ok(RecentState {
        recent_ids: config.recent_ids.clone(),
        recent_meta: config.recent_meta.clone(),
    })
}

#[tauri::command]
fn set_recent_enabled(
    app: AppHandle,
    state: State<Arc<AppState>>,
    recent_enabled: bool,
) -> Result<(), String> {
    let mut config = state.config.lock().unwrap();
    config.recent_enabled = recent_enabled;
    save(&app, &config)
}

#[tauri::command]
fn set_top_tags_scope(
    app: AppHandle,
    state: State<Arc<AppState>>,
    use_results: bool,
) -> Result<(), String> {
    let mut config = state.config.lock().unwrap();
    config.top_tags_use_results = use_results;
    save(&app, &config)
}

#[tauri::command]
fn set_top_tags_limit(
    app: AppHandle,
    state: State<Arc<AppState>>,
    limit: u32,
) -> Result<(), String> {
    let mut config = state.config.lock().unwrap();
    let value = limit.clamp(1, 20);
    config.top_tags_limit = value;
    save(&app, &config)
}

#[tauri::command]
fn set_show_shortcuts_hint(
    app: AppHandle,
    state: State<Arc<AppState>>,
    show_shortcuts_hint: bool,
) -> Result<(), String> {
    let mut config = state.config.lock().unwrap();
    config.show_shortcuts_hint = show_shortcuts_hint;
    save(&app, &config)
}

#[tauri::command]
fn clear_recent(
    app: AppHandle,
    state: State<Arc<AppState>>,
) -> Result<RecentState, String> {
    let mut config = state.config.lock().unwrap();
    config.recent_ids.clear();
    config.recent_meta.clear();
    save(&app, &config)?;
    Ok(RecentState {
        recent_ids: config.recent_ids.clone(),
        recent_meta: config.recent_meta.clone(),
    })
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
    let pending = { state.pending_paths.lock().unwrap().clone() };
    let mut next_pending = HashSet::new();
    let mut visible = Vec::new();

    for prompt in prompts {
        if pending.contains(&prompt.id) {
            let size = fs::metadata(&prompt.path).map(|m| m.len()).unwrap_or(0);
            if size == 0 {
                next_pending.insert(prompt.id.clone());
                continue;
            }
        }
        visible.push(prompt);
    }

    if !pending.is_empty() {
        *state.pending_paths.lock().unwrap() = next_pending;
    }

    let mut lock = state.prompts.write().unwrap();
    *lock = visible.clone();
    visible
}

fn build_prompt_tag_map(state: &Arc<AppState>) -> HashMap<String, Vec<String>> {
    let prompts = state.prompts.read().unwrap();
    let mut map = HashMap::new();
    for prompt in prompts.iter() {
        map.insert(prompt.id.clone(), prompt.tags.clone());
    }
    map
}

fn normalize_input_tags(raw: Vec<String>) -> Result<Vec<String>, String> {
    let mut tags = Vec::new();
    let mut seen = HashSet::new();
    for value in raw {
        for token in value.split_whitespace() {
            let token = token.trim_start_matches('#');
            if token.is_empty() {
                continue;
            }
            let normalized = normalize_tag(token)
                .ok_or_else(|| "标签仅允许中英文数字，长度 1-10".to_string())?;
            if seen.insert(normalized.clone()) {
                tags.push(normalized);
            }
        }
    }
    Ok(tags)
}

fn is_valid_filename(name: &str) -> bool {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return false;
    }
    if trimmed.ends_with('.') || trimmed.ends_with(' ') {
        return false;
    }
    if trimmed == "." || trimmed == ".." {
        return false;
    }
    let invalid = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    if trimmed.chars().any(|ch| invalid.contains(&ch)) {
        return false;
    }
    true
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
    let show = MenuItem::with_id(app, "show", "显示", true, None::<&str>)
        .map_err(|e| format!("menu item failed: {e}"))?;
    let hide = MenuItem::with_id(app, "hide", "隐藏", true, None::<&str>)
        .map_err(|e| format!("menu item failed: {e}"))?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)
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
            if let TrayIconEvent::Click {
                button,
                button_state,
                ..
            } = event
            {
                if button == MouseButton::Left && button_state == MouseButtonState::Up {
                    let app = tray.app_handle();
                    let state = app.state::<Arc<AppState>>();
                    let _ = store_active_window(state.inner());
                    let _ = toggle_main_window(&app);
                }
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

    let quickstart = dir.join("快速开始 #welcome.md");
    if !quickstart.exists() {
        let content = "\
提示词启动器快速开始

- 使用全局快捷键打开启动器。
- 输入关键词搜索，支持 #标签。
- Enter 粘贴，右键打开文件。
";
        fs::write(&quickstart, content).map_err(|e| format!("seed quickstart failed: {e}"))?;
    }

    let examples_dir = dir.join("Examples");
    fs::create_dir_all(&examples_dir).map_err(|e| format!("seed dir failed: {e}"))?;
    let email = examples_dir.join("邮件回复 #email.txt");
    if !email.exists() {
        let content = "\
请用友好、简洁的语气回复。
总结对方需求，确认下一步，并询问缺失信息。
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
    let mut watcher = notify::recommended_watcher(
        move |res: notify::Result<notify::Event>| {
            if res.is_err() {
                return;
            }
            let prompts = refresh_prompts(&state_handle, &index_dir);
            let _ = app_handle.emit("prompts-updated", prompts);
        },
    )
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
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let handle = app.handle();
            let config = load_or_init(handle)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

            let dir = PathBuf::from(&config.prompts_dir);
            fs::create_dir_all(&dir)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            seed_prompts_if_empty(&dir)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            let _ = std::env::current_exe()
                .map_err(|e| format!("resolve exe path failed: {e}"))
                .and_then(|exe| autostart::set_auto_start(config.auto_start, &exe));

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
            if cfg!(debug_assertions) {
                let _ = show_main_window(&handle);
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            list_prompts,
            search_prompts,
            set_prompts_dir,
            create_prompt_file,
            open_prompt_path,
            set_auto_paste,
            set_hotkey,
            set_auto_start,
            toggle_favorite,
            push_recent,
            set_recent_enabled,
            update_prompt_tags,
            set_top_tags_scope,
            set_top_tags_limit,
            set_show_shortcuts_hint,
            clear_recent,
            capture_active_window,
            focus_last_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
