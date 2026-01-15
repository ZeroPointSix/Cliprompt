use serde::Serialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize)]
pub struct PromptEntry {
    pub id: String,
    pub title: String,
    pub body: String,
    pub preview: String,
    pub tags: Vec<String>,
    pub path: String,
}

pub fn index_prompts(dir: &Path) -> Vec<PromptEntry> {
    let mut entries = Vec::new();
    for entry in WalkDir::new(dir).follow_links(true).into_iter().flatten() {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if !is_prompt_file(path) {
            continue;
        }
        if let Some(prompt) = read_prompt(path, dir) {
            entries.push(prompt);
        }
    }
    entries
}

fn read_prompt(path: &Path, root: &Path) -> Option<PromptEntry> {
    let body = fs::read_to_string(path).ok()?;
    let title = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let mut tags: HashSet<String> = extract_tags(&title).into_iter().collect();
    for tag in extract_path_tags(path, root) {
        tags.insert(tag);
    }
    let mut tags: Vec<String> = tags.into_iter().collect();
    tags.sort();
    let preview = make_preview(&body);
    let path_string = path.to_string_lossy().to_string();

    Some(PromptEntry {
        id: path_string.clone(),
        title,
        body,
        preview,
        tags,
        path: path_string,
    })
}

fn is_prompt_file(path: &Path) -> bool {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) => matches!(ext.to_ascii_lowercase().as_str(), "md" | "txt"),
        None => false,
    }
}

fn make_preview(body: &str) -> String {
    let first_line = body
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("")
        .trim();
    let mut preview = first_line.replace('\t', " ");
    if preview.len() > 120 {
        preview.truncate(120);
        preview.push_str("...");
    }
    preview
}

fn extract_tags(title: &str) -> Vec<String> {
    let mut tags = HashSet::new();

    let mut rest = title;
    while let Some(start) = rest.find('[') {
        if let Some(end) = rest[start + 1..].find(']') {
            let tag = &rest[start + 1..start + 1 + end];
            if let Some(normalized) = normalize_tag(tag) {
                tags.insert(normalized);
            }
            rest = &rest[start + 1 + end + 1..];
        } else {
            break;
        }
    }

    for token in title.split_whitespace() {
        if let Some(tag) = token.strip_prefix('#') {
            if let Some(normalized) = normalize_tag(tag) {
                tags.insert(normalized);
            }
        }
    }

    tags.into_iter().collect()
}

fn extract_path_tags(path: &Path, root: &Path) -> Vec<String> {
    let mut tags = Vec::new();
    let Ok(relative) = path.strip_prefix(root) else {
        return tags;
    };
    let Some(parent) = relative.parent() else {
        return tags;
    };
    for component in parent.components() {
        let name = component.as_os_str().to_string_lossy();
        if let Some(tag) = normalize_tag(&name) {
            tags.push(tag);
        }
    }
    tags
}

fn normalize_tag(raw: &str) -> Option<String> {
    let trimmed = raw.trim_matches(|c: char| !c.is_alphanumeric() && c != '-' && c != '_');
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_ascii_lowercase())
    }
}
