use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const TAGS_META_FILENAME: &str = ".tags.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TagsMeta {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub updated_at: i64,
    pub tags_by_path: HashMap<String, Vec<String>>,
}

impl Default for TagsMeta {
    fn default() -> Self {
        Self {
            version: default_version(),
            updated_at: 0,
            tags_by_path: HashMap::new(),
        }
    }
}

fn default_version() -> u32 {
    1
}

pub fn tags_meta_path(root: &Path) -> PathBuf {
    root.join(TAGS_META_FILENAME)
}

pub fn load_tags_meta(root: &Path) -> Result<TagsMeta, String> {
    let path = tags_meta_path(root);
    if !path.exists() {
        return Ok(TagsMeta::default());
    }
    let data = fs::read_to_string(&path)
        .map_err(|e| format!("read tags meta failed: {e}"))?;
    let meta: TagsMeta = serde_json::from_str(&data)
        .map_err(|e| format!("parse tags meta failed: {e}"))?;
    Ok(meta)
}

pub fn save_tags_meta(root: &Path, meta: &TagsMeta) -> Result<(), String> {
    let path = tags_meta_path(root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("create tags meta dir failed: {e}"))?;
    }
    let data = serde_json::to_string_pretty(meta)
        .map_err(|e| format!("serialize tags meta failed: {e}"))?;
    fs::write(&path, data).map_err(|e| format!("write tags meta failed: {e}"))?;
    Ok(())
}

pub fn resolve_tags_for_path(
    meta: &TagsMeta,
    root: &Path,
    path: &Path,
    fallback: Vec<String>,
) -> Vec<String> {
    let key = path_to_key(root, path);
    if let Some(tags) = meta.tags_by_path.get(&key) {
        return tags.clone();
    }
    fallback
}

pub fn path_to_key(root: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(root).unwrap_or(path);
    relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

pub fn touch_updated_at(meta: &mut TagsMeta) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    meta.updated_at = now;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn make_temp_dir(prefix: &str) -> PathBuf {
        let mut dir = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        dir.push(format!("{prefix}-{nanos}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn load_missing_returns_default() {
        let dir = make_temp_dir("tags-meta-missing");
        let meta = load_tags_meta(&dir).expect("load tags meta");
        assert_eq!(meta, TagsMeta::default());
    }

    #[test]
    fn save_then_load_roundtrip() {
        let dir = make_temp_dir("tags-meta-roundtrip");
        let mut meta = TagsMeta::default();
        meta.version = 1;
        meta.updated_at = 123;
        meta.tags_by_path
            .insert("foo.txt".to_string(), vec!["tag1".to_string()]);

        save_tags_meta(&dir, &meta).expect("save tags meta");
        let loaded = load_tags_meta(&dir).expect("load tags meta");
        assert_eq!(loaded, meta);
    }

    #[test]
    fn resolve_prefers_meta_over_fallback() {
        let dir = make_temp_dir("tags-meta-resolve");
        let mut meta = TagsMeta::default();
        meta.tags_by_path
            .insert("foo.txt".to_string(), vec!["a".to_string()]);

        let path = dir.join("foo.txt");
        let resolved =
            resolve_tags_for_path(&meta, &dir, &path, vec!["b".to_string()]);
        assert_eq!(resolved, vec!["a".to_string()]);
    }

    #[test]
    fn resolve_uses_fallback_when_missing() {
        let dir = make_temp_dir("tags-meta-fallback");
        let meta = TagsMeta::default();
        let path = dir.join("bar.txt");
        let resolved =
            resolve_tags_for_path(&meta, &dir, &path, vec!["b".to_string()]);
        assert_eq!(resolved, vec!["b".to_string()]);
    }
}
