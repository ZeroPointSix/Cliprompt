mod config;
mod domain;
mod infrastructure;
mod lifecycle;
mod prompts;
mod tags_meta;
mod usecase;

#[cfg(target_os = "windows")]
mod autostart;
#[cfg(target_os = "windows")]
mod win;

use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use std::time::Duration;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, State,
};
use tauri::Emitter;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
use tauri_plugin_opener::OpenerExt;

use config::{load_or_init, save, AppConfig};
use infrastructure::fs_prompt_file_repository::FsPromptFileRepository;
use lifecycle::{GateDecision, LauncherGate};
use prompts::{
    index_prompts, make_preview, normalize_tag, search_prompts as search_prompts_impl, PromptEntry,
};
use tags_meta::{load_tags_meta, path_to_key, save_tags_meta, touch_updated_at};
use usecase::create_prompt_file::CreatePromptFileUseCase;

const PREVIEW_CHARS_MIN: u32 = 10;
const PREVIEW_CHARS_MAX: u32 = 200;
// Grace period to keep newly created empty prompts hidden until editors finish saving.
const PENDING_PROMPT_TTL_MS: u128 = 5_000;
const EVENT_LAUNCHER_SHOWN: &str = "launcher-shown";

struct AppState {
    prompts: RwLock<Vec<PromptEntry>>,
    config: Mutex<AppConfig>,
    watcher: Mutex<Option<RecommendedWatcher>>,
    last_active_hwnd: Mutex<Option<isize>>,
    pending_paths: Mutex<HashMap<String, u128>>,
    registered_hotkey: Mutex<Option<String>>,
    launcher_gate: Mutex<LauncherGate>,
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
            pending_paths: Mutex::new(HashMap::new()),
            registered_hotkey: Mutex::new(None),
            launcher_gate: Mutex::new(LauncherGate::new()),
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
fn frontend_ready(app: AppHandle, state: State<Arc<AppState>>) -> Result<(), String> {
    let should_show = state.launcher_gate.lock().unwrap().set_ui_ready();
    if should_show {
        show_main_window(&app)?;
    }
    Ok(())
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
    let dir = {
        let config = state.config.lock().unwrap();
        config.prompts_dir.clone()
    };
    if dir.trim().is_empty() {
        return Err("提示词目录未配置".to_string());
    }
    let root = PathBuf::from(dir);
    let usecase = CreatePromptFileUseCase::new(FsPromptFileRepository);
    let path = usecase.execute(&root, &name)?;
    let path_string = path.to_string_lossy().to_string();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("time error: {e}"))?
        .as_millis();
    state
        .pending_paths
        .lock()
        .unwrap()
        .insert(path_string.clone(), now);
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
fn delete_prompt_files(
    app: AppHandle,
    state: State<Arc<AppState>>,
    paths: Vec<String>,
) -> Result<Vec<PromptEntry>, String> {
    if paths.is_empty() {
        return Err("未选择任何提示词".to_string());
    }
    let root = resolve_prompts_root(state.inner())?;
    let mut remove_ids = HashSet::new();
    let mut remove_keys = Vec::new();

    for raw_path in &paths {
        let target = resolve_prompt_path(&root, Path::new(raw_path))?;
        let target_str = target.to_string_lossy().to_string();
        if target.exists() {
            fs::remove_file(&target).map_err(|e| format!("删除失败: {e}"))?;
        }
        remove_ids.insert(raw_path.clone());
        remove_ids.insert(target_str.clone());
        remove_keys.push(path_to_key(&root, &target));
    }

    {
        let mut pending = state.pending_paths.lock().unwrap();
        pending.retain(|item, _| !remove_ids.contains(item));
    }

    {
        let mut config = state.config.lock().unwrap();
        config.favorites.retain(|item| !remove_ids.contains(item));
        config.recent_ids.retain(|item| !remove_ids.contains(item));
        config.recent_meta.retain(|key, _| !remove_ids.contains(key));
        save(&app, &config)?;
    }

    if let Ok(mut meta) = load_tags_meta(&root) {
        let mut changed = false;
        for key in remove_keys {
            if meta.tags_by_path.remove(&key).is_some() {
                changed = true;
            }
        }
        if changed {
            touch_updated_at(&mut meta);
            let _ = save_tags_meta(&root, &meta);
        }
    }

    Ok(refresh_prompts(state.inner(), &root))
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
fn set_append_clipboard(
    app: AppHandle,
    state: State<Arc<AppState>>,
    append_clipboard: bool,
) -> Result<(), String> {
    let mut config = state.config.lock().unwrap();
    config.append_clipboard = append_clipboard;
    save(&app, &config)
}

#[tauri::command]
fn set_hotkey(
    app: AppHandle,
    state: State<Arc<AppState>>,
    hotkey: String,
) -> Result<(), String> {
    let trimmed = hotkey.trim();
    if trimmed.is_empty() {
        return Err("快捷键不能为空".to_string());
    }
    update_hotkey_registration(&app, state.inner(), trimmed)?;
    let mut config = state.config.lock().unwrap();
    if config.hotkey == trimmed {
        return Ok(());
    }
    config.hotkey = trimmed.to_string();
    save(&app, &config)
}

#[tauri::command]
fn set_auto_start(
    app: AppHandle,
    state: State<Arc<AppState>>,
    auto_start: bool,
) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let exe_path = std::env::current_exe()
            .map_err(|e| format!("resolve exe path failed: {e}"))?;
        autostart::set_auto_start(auto_start, &exe_path)?;
    }

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
fn set_preview_chars(
    app: AppHandle,
    state: State<Arc<AppState>>,
    preview_chars: u32,
) -> Result<(), String> {
    let value = preview_chars.clamp(PREVIEW_CHARS_MIN, PREVIEW_CHARS_MAX);
    {
        let mut config = state.config.lock().unwrap();
        config.preview_chars = value;
        save(&app, &config)?;
    }
    let preview_chars = value as usize;
    let mut prompts = state.prompts.write().unwrap();
    for prompt in prompts.iter_mut() {
        prompt.preview = make_preview(&prompt.body, preview_chars);
    }
    Ok(())
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
    #[cfg(target_os = "windows")]
    {
        store_active_window(state.inner())
    }
    #[cfg(not(target_os = "windows"))]
    {
        Ok(())
    }
}

#[tauri::command]
fn focus_last_window(
    state: State<Arc<AppState>>,
    auto_paste: bool,
) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
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
    #[cfg(not(target_os = "windows"))]
    {
        Ok(())
    }
}

fn refresh_prompts(state: &Arc<AppState>, dir: &Path) -> Vec<PromptEntry> {
    let preview_chars = {
        let config = state.config.lock().unwrap();
        config
            .preview_chars
            .clamp(PREVIEW_CHARS_MIN, PREVIEW_CHARS_MAX) as usize
    };
    let prompts = index_prompts(dir, preview_chars);
    let pending = { state.pending_paths.lock().unwrap().clone() };
    let mut next_pending = HashMap::new();
    let mut visible = Vec::new();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_millis())
        .unwrap_or(0);

    for prompt in prompts {
        if let Some(created_at) = pending.get(&prompt.id) {
            let size = fs::metadata(&prompt.path).map(|m| m.len()).unwrap_or(0);
            let age = now.saturating_sub(*created_at);
            if size == 0 && age < PENDING_PROMPT_TTL_MS {
                next_pending.insert(prompt.id.clone(), *created_at);
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

fn store_active_window(state: &Arc<AppState>) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        if let Some(hwnd) = win::capture_foreground_window() {
            *state.last_active_hwnd.lock().unwrap() = Some(hwnd);
            Ok(())
        } else {
            Err("no active window detected".to_string())
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        Ok(())
    }
}

fn register_global_hotkey(
    app: &AppHandle,
    state: &Arc<AppState>,
    hotkey: &str,
) -> Result<(), String> {
    let state_handle = state.clone();
    app.global_shortcut()
        .on_shortcut(hotkey, move |app_handle, _, event| {
            if event.state == ShortcutState::Pressed {
                let _ = request_toggle(app_handle, &state_handle);
            }
        })
        .map_err(|e| format!("register hotkey failed: {e}"))
}

fn update_hotkey_registration(
    app: &AppHandle,
    state: &Arc<AppState>,
    hotkey: &str,
) -> Result<(), String> {
    let current = state.registered_hotkey.lock().unwrap().clone();
    if current.as_deref() == Some(hotkey) {
        return Ok(());
    }

    register_global_hotkey(app, state, hotkey)?;

    if let Some(previous) = current {
        let _ = app.global_shortcut().unregister(previous.as_str());
    }

    *state.registered_hotkey.lock().unwrap() = Some(hotkey.to_string());
    Ok(())
}

fn request_show(app: &AppHandle, state: &Arc<AppState>) -> Result<(), String> {
    let should_show = state.launcher_gate.lock().unwrap().request_show();
    if !should_show {
        let _ = store_active_window(state);
        return Ok(());
    }
    let _ = store_active_window(state);
    show_main_window(app)
}

fn request_toggle(app: &AppHandle, state: &Arc<AppState>) -> Result<(), String> {
    let decision = state
        .launcher_gate
        .lock()
        .unwrap()
        .allow_toggle(Instant::now());
    match decision {
        GateDecision::Debounced => Ok(()),
        GateDecision::DeferShow => {
            let _ = store_active_window(state);
            Ok(())
        }
        GateDecision::Proceed => toggle_main_window(app, state),
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
    let _ = window.emit(EVENT_LAUNCHER_SHOWN, ());
    Ok(())
}

fn hide_main_window(app: &AppHandle) -> Result<(), String> {
    let window = main_window(app)?;
    window
        .hide()
        .map_err(|e| format!("hide window failed: {e}"))?;
    Ok(())
}

fn toggle_main_window(app: &AppHandle, state: &Arc<AppState>) -> Result<(), String> {
    let window = main_window(app)?;
    let visible = window
        .is_visible()
        .map_err(|e| format!("query window failed: {e}"))?;
    if visible {
        hide_main_window(app)
    } else {
        let _ = store_active_window(state);
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
                let _ = request_show(app, state.inner());
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
                    let _ = request_toggle(&app, state.inner());
                }
            }
        })
        .build(app)
        .map_err(|e| format!("tray init failed: {e}"))?;

    Ok(())
}

fn seed_prompts_if_empty(dir: &Path, preview_chars: usize) -> Result<(), String> {
    if !dir.exists() {
        return Ok(());
    }
    if !index_prompts(dir, preview_chars).is_empty() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn make_temp_dir(prefix: &str) -> std::path::PathBuf {
        let mut dir = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        dir.push(format!("{prefix}-{nanos}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn make_state(dir: &Path) -> Arc<AppState> {
        let mut config = AppConfig::default();
        config.prompts_dir = dir.to_string_lossy().to_string();
        Arc::new(AppState::new(config))
    }

    #[test]
    fn normalize_input_tags_dedupes_and_normalizes() {
        let tags = normalize_input_tags(vec![
            "#Tag1 tag1 标签2".to_string(),
            "foo #foo".to_string(),
        ])
        .unwrap();
        assert_eq!(
            tags,
            vec![
                "tag1".to_string(),
                "标签2".to_string(),
                "foo".to_string()
            ]
        );
    }

    #[test]
    fn normalize_input_tags_rejects_invalid() {
        assert!(normalize_input_tags(vec!["tag-1".to_string()]).is_err());
    }

    #[test]
    fn refresh_prompts_hides_pending_empty_within_ttl() {
        let dir = make_temp_dir("pending-hide");
        let path = dir.join("empty.txt");
        fs::write(&path, "").unwrap();
        let state = make_state(&dir);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        state
            .pending_paths
            .lock()
            .unwrap()
            .insert(path.to_string_lossy().to_string(), now);

        let results = refresh_prompts(&state, &dir);
        assert!(results.is_empty());
    }

    #[test]
    fn refresh_prompts_shows_pending_empty_after_ttl() {
        let dir = make_temp_dir("pending-show");
        let path = dir.join("empty.txt");
        fs::write(&path, "").unwrap();
        let state = make_state(&dir);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let old = now.saturating_sub(PENDING_PROMPT_TTL_MS + 1);
        state
            .pending_paths
            .lock()
            .unwrap()
            .insert(path.to_string_lossy().to_string(), old);

        let results = refresh_prompts(&state, &dir);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].path, path.to_string_lossy().to_string());
    }
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
            let preview_chars =
                config.preview_chars.clamp(PREVIEW_CHARS_MIN, PREVIEW_CHARS_MAX) as usize;

            let dir = PathBuf::from(&config.prompts_dir);
            fs::create_dir_all(&dir)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            seed_prompts_if_empty(&dir, preview_chars)
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
            if let Err(error) = update_hotkey_registration(&handle, &state, &hotkey) {
                eprintln!("[hotkey] register failed: {error}");
            }
            init_tray(app)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            start_watcher(handle.clone(), state.clone(), dir)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            if cfg!(debug_assertions) {
                let _ = request_show(&handle, &state);
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
