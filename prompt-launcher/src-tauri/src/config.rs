use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri::{path::BaseDirectory, AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub prompts_dir: String,
    pub auto_paste: bool,
    pub hotkey: String,
    #[serde(default)]
    pub auto_start: bool,
    #[serde(default)]
    pub favorites: Vec<String>,
    #[serde(default)]
    pub recent_ids: Vec<String>,
    #[serde(default)]
    pub recent_enabled: bool,
    #[serde(default)]
    pub recent_meta: HashMap<String, i64>,
    #[serde(default)]
    pub top_tags_use_results: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            prompts_dir: String::new(),
            auto_paste: true,
            hotkey: "Alt+Space".to_string(),
            auto_start: false,
            favorites: Vec::new(),
            recent_ids: Vec::new(),
            recent_enabled: true,
            recent_meta: HashMap::new(),
            top_tags_use_results: false,
        }
    }
}

pub fn load_or_init(app: &AppHandle) -> Result<AppConfig, String> {
    let path = config_path(app)?;
    if !path.exists() {
        let mut config = AppConfig::default();
        config.prompts_dir = default_prompts_dir(app)?
            .to_string_lossy()
            .to_string();
        save(app, &config)?;
        return Ok(config);
    }

    let data = fs::read_to_string(&path)
        .map_err(|e| format!("read config failed: {e}"))?;
    let mut config: AppConfig =
        serde_json::from_str(&data).map_err(|e| format!("parse config failed: {e}"))?;

    if config.prompts_dir.trim().is_empty() {
        config.prompts_dir = default_prompts_dir(app)?
            .to_string_lossy()
            .to_string();
        save(app, &config)?;
    }

    Ok(config)
}

pub fn save(app: &AppHandle, config: &AppConfig) -> Result<(), String> {
    let path = config_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("create config dir failed: {e}"))?;
    }

    let data =
        serde_json::to_string_pretty(config).map_err(|e| format!("serialize config failed: {e}"))?;
    fs::write(&path, data).map_err(|e| format!("write config failed: {e}"))?;
    Ok(())
}

fn config_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .resolve("config.json", BaseDirectory::AppConfig)
        .map_err(|e| format!("resolve config path failed: {e}"))
}

fn default_prompts_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .resolve("PromptLauncher/Prompts", BaseDirectory::Document)
        .map_err(|e| format!("resolve prompts dir failed: {e}"))
}
