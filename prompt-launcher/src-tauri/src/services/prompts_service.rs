use crate::config::save;
use crate::constants::EVENT_PROMPTS_UPDATED;
use crate::infrastructure::fs_prompt_file_repository::FsPromptFileRepository;
use crate::prompts::{
    index_prompts, make_preview, normalize_tag, search_prompts as search_prompts_impl, PromptEntry,
};
use crate::state::AppState;
use crate::tags_meta::{load_tags_meta, path_to_key, save_tags_meta, touch_updated_at};
use crate::usecase::create_prompt_file::CreatePromptFileUseCase;
use notify::{RecursiveMode, Watcher};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Emitter;
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

pub const PREVIEW_CHARS_MIN: u32 = 10;
pub const PREVIEW_CHARS_MAX: u32 = 200;
// Grace period to keep newly created empty prompts hidden until editors finish saving.
const PENDING_PROMPT_TTL_MS: u128 = 5_000;

pub struct PromptsService;

impl PromptsService {
    pub fn list(state: &AppState) -> Vec<PromptEntry> {
        state.prompts.read().unwrap().clone()
    }

    pub fn search(
        state: &AppState,
        query: &str,
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
            return search_prompts_impl(&filtered, query, limit);
        }
        search_prompts_impl(&prompts, query, limit)
    }

    pub fn set_prompts_dir(
        app: &AppHandle,
        state: &Arc<AppState>,
        path: String,
    ) -> Result<Vec<PromptEntry>, String> {
        let dir = PathBuf::from(&path);
        fs::create_dir_all(&dir).map_err(|e| format!("create prompts dir failed: {e}"))?;

        {
            let mut config = state.config.lock().unwrap();
            config.prompts_dir = dir.to_string_lossy().to_string();
            save(app, &config)?;
        }

        {
            let mut pending = state.pending_paths.lock().unwrap();
            pending.clear();
        }

        let prompts = Self::refresh_prompts(state, &dir);
        Self::start_watcher(app.clone(), state.clone(), dir)?;
        Ok(prompts)
    }

    pub fn create_prompt_file(state: &Arc<AppState>, name: String) -> Result<String, String> {
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

    pub fn open_prompt_path(
        app: &AppHandle,
        state: &AppState,
        path: String,
    ) -> Result<(), String> {
        let root = resolve_prompts_root(state)?;
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

    pub fn delete_prompt_files(
        app: &AppHandle,
        state: &Arc<AppState>,
        paths: Vec<String>,
    ) -> Result<Vec<PromptEntry>, String> {
        if paths.is_empty() {
            return Err("未选择任何提示词".to_string());
        }
        let root = resolve_prompts_root(state)?;
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
            save(app, &config)?;
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

        Ok(Self::refresh_prompts(state, &root))
    }

    pub fn update_prompt_tags(
        state: &Arc<AppState>,
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
        let prompt_map = build_prompt_tag_map(state);

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
        Ok(Self::refresh_prompts(state, &root))
    }

    pub fn refresh_prompts(state: &Arc<AppState>, dir: &Path) -> Vec<PromptEntry> {
        let preview_chars = {
            let config = state.config.lock().unwrap();
            Self::clamp_preview_chars(config.preview_chars) as usize
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

    pub fn apply_preview_chars(state: &Arc<AppState>, preview_chars: u32) {
        let preview_chars = Self::clamp_preview_chars(preview_chars) as usize;
        let mut prompts = state.prompts.write().unwrap();
        for prompt in prompts.iter_mut() {
            prompt.preview = make_preview(&prompt.body, preview_chars);
        }
    }

    pub fn seed_prompts_if_empty(dir: &Path, preview_chars: usize) -> Result<(), String> {
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

    pub fn start_watcher(
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
                let prompts = Self::refresh_prompts(&state_handle, &index_dir);
                let _ = app_handle.emit(EVENT_PROMPTS_UPDATED, prompts);
            },
        )
        .map_err(|e| format!("watcher init failed: {e}"))?;

        watcher
            .watch(&dir, RecursiveMode::Recursive)
            .map_err(|e| format!("watcher start failed: {e}"))?;

        *state.watcher.lock().unwrap() = Some(watcher);
        Ok(())
    }

    pub fn clamp_preview_chars(value: u32) -> u32 {
        value.clamp(PREVIEW_CHARS_MIN, PREVIEW_CHARS_MAX)
    }
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

fn build_prompt_tag_map(state: &AppState) -> HashMap<String, Vec<String>> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
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
            vec!["tag1".to_string(), "标签2".to_string(), "foo".to_string()]
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

        let results = PromptsService::refresh_prompts(&state, &dir);
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

        let results = PromptsService::refresh_prompts(&state, &dir);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].path, path.to_string_lossy().to_string());
    }
}
