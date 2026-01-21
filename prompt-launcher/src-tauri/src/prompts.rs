use serde::Serialize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

use crate::tags_meta::{load_tags_meta, resolve_tags_for_path, TagsMeta};

#[derive(Debug, Clone, Serialize)]
pub struct PromptEntry {
    pub id: String,
    pub title: String,
    pub body: String,
    pub preview: String,
    pub tags: Vec<String>,
    pub path: String,
}

pub fn search_prompts(
    prompts: &[PromptEntry],
    query: &str,
    limit: usize,
) -> Vec<PromptEntry> {
    let trimmed = query.trim().to_lowercase();
    if trimmed.is_empty() {
        return prompts.iter().take(limit).cloned().collect();
    }

    let (tags, terms) = split_query(&trimmed);
    let mut results: Vec<(i32, PromptEntry)> = Vec::new();

    for prompt in prompts {
        if !tags.is_empty() && !tags_match(prompt, &tags) {
            continue;
        }

        let score = if terms.is_empty() {
            Some(0)
        } else {
            score_prompt(prompt, &terms)
        };

        if let Some(score) = score {
            results.push((score, prompt.clone()));
        }
    }

    results.sort_by_key(|(score, _)| *score);
    results
        .into_iter()
        .take(limit)
        .map(|(_, prompt)| prompt)
        .collect()
}

pub fn index_prompts(dir: &Path, preview_chars: usize) -> Vec<PromptEntry> {
    let meta = match load_tags_meta(dir) {
        Ok(meta) => meta,
        Err(err) => {
            eprintln!("[tags_meta] load failed: {err}");
            TagsMeta::default()
        }
    };
    let mut entries = Vec::new();
    for entry in WalkDir::new(dir).follow_links(true).into_iter().flatten() {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if !is_prompt_file(path) {
            continue;
        }
        if let Some(prompt) = read_prompt(path, dir, &meta, preview_chars) {
            entries.push(prompt);
        }
    }
    entries
}

fn read_prompt(
    path: &Path,
    root: &Path,
    meta: &TagsMeta,
    preview_chars: usize,
) -> Option<PromptEntry> {
    let body = fs::read_to_string(path).ok()?;
    let title = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let mut fallback_tags: HashSet<String> = extract_tags(&title).into_iter().collect();
    for tag in extract_path_tags(path, root) {
        fallback_tags.insert(tag);
    }
    let fallback: Vec<String> = fallback_tags.into_iter().collect();
    let resolved = resolve_tags_for_path(meta, root, path, fallback);
    let mut tags = normalize_tags(resolved);
    tags.sort();
    let preview = make_preview(&body, preview_chars);
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

pub(crate) fn make_preview(body: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    let mut preview = String::new();
    let mut count = 0usize;
    let mut truncated = false;

    for token in body.split_whitespace() {
        if count >= max_chars {
            truncated = true;
            break;
        }
        if !preview.is_empty() {
            if count + 1 > max_chars {
                truncated = true;
                break;
            }
            preview.push(' ');
            count += 1;
        }
        for ch in token.chars() {
            if count >= max_chars {
                truncated = true;
                break;
            }
            preview.push(ch);
            count += 1;
        }
        if truncated {
            break;
        }
    }

    if truncated {
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

pub(crate) fn normalize_tag(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.chars().count() > 10 {
        return None;
    }
    if !trimmed.chars().all(is_allowed_tag_char) {
        return None;
    }
    Some(trimmed.to_ascii_lowercase())
}

fn is_allowed_tag_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || (ch >= '\u{4E00}' && ch <= '\u{9FFF}')
}

fn normalize_tags(raw: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut tags = Vec::new();
    for tag in raw {
        if let Some(normalized) = normalize_tag(&tag) {
            if seen.insert(normalized.clone()) {
                tags.push(normalized);
            }
        }
    }
    tags
}

fn split_query(query: &str) -> (Vec<String>, Vec<String>) {
    let mut tags = Vec::new();
    let mut terms = Vec::new();

    for token in query.split_whitespace() {
        if let Some(tag) = token.strip_prefix('#') {
            if let Some(normalized) = normalize_tag(tag) {
                tags.push(normalized);
            }
        } else {
            terms.push(token.to_string());
        }
    }

    (tags, terms)
}

fn tags_match(prompt: &PromptEntry, tags: &[String]) -> bool {
    tags.iter().all(|tag| prompt.tags.iter().any(|t| t == tag))
}

fn score_prompt(prompt: &PromptEntry, terms: &[String]) -> Option<i32> {
    let tag_text = prompt.tags.join(" ").to_lowercase();
    let title_text = prompt.title.to_lowercase();
    let full_text = format!(
        "{} {} {} {}",
        prompt.title, prompt.preview, prompt.body, tag_text
    )
    .to_lowercase();

    let mut best = score_terms(&full_text, terms)?;
    // Prioritize title matches with much higher weight (-10000)
    // This ensures title matches always appear before content matches
    if let Some(score) = score_terms(&title_text, terms) {
        best = best.min(score - 10000);
    }
    if let Some(score) = score_terms(&tag_text, terms) {
        best = best.min(score - 80);
    }
    Some(best)
}

fn score_terms(text: &str, terms: &[String]) -> Option<i32> {
    let mut total = 0;
    for term in terms {
        total += score_match(text, term)?;
    }
    Some(total)
}

fn score_match(text: &str, term: &str) -> Option<i32> {
    if term.is_empty() {
        return Some(0);
    }

    let mut score: i32 = 0;
    if let Some(best_index) = best_substring_index(text, term) {
        score -= 200 + best_index as i32;
        if is_word_boundary(text, best_index) {
            score -= 30;
        }
        let end = best_index + term.len();
        if is_word_boundary(text, end) {
            score -= 10;
        }
    }

    let mut last_pos: i32 = -1;
    let mut start = 0usize;

    for ch in term.chars() {
        let slice = &text[start..];
        let mut found = None;
        for (pos, candidate) in slice.char_indices() {
            if candidate == ch {
                found = Some(start + pos);
                break;
            }
        }
        let next_index = found?;
        score += next_index as i32 - last_pos;
        last_pos = next_index as i32;
        start = next_index + ch.len_utf8();
    }

    Some(score)
}

fn best_substring_index(text: &str, term: &str) -> Option<usize> {
    text.find(term)
}

fn is_word_boundary(text: &str, index: usize) -> bool {
    if index == 0 || index >= text.len() {
        return true;
    }
    let prev = text[..index].chars().last();
    let next = text[index..].chars().next();
    let prev_is_word = prev.map(|c| c.is_alphanumeric()).unwrap_or(false);
    let next_is_word = next.map(|c| c.is_alphanumeric()).unwrap_or(false);
    !prev_is_word || !next_is_word
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

    #[test]
    fn normalize_tag_accepts_chinese_and_ascii() {
        assert_eq!(normalize_tag("Tag1"), Some("tag1".to_string()));
        assert_eq!(normalize_tag("标签1"), Some("标签1".to_string()));
    }

    #[test]
    fn normalize_tag_rejects_special_chars() {
        assert_eq!(normalize_tag("tag-1"), None);
        assert_eq!(normalize_tag("tag_1"), None);
        assert_eq!(normalize_tag("tag!"), None);
        assert_eq!(normalize_tag("tag 1"), None);
    }

    #[test]
    fn normalize_tag_rejects_overlong() {
        assert_eq!(normalize_tag("12345678901"), None);
    }

    #[test]
    fn split_query_filters_invalid_tags() {
        let (tags, terms) = split_query("#Tag #tag-1 foo");
        assert_eq!(tags, vec!["tag".to_string()]);
        assert_eq!(terms, vec!["foo".to_string()]);
    }

    #[test]
    fn tags_match_and_logic() {
        let dir = make_temp_dir("tags-match");
        let path = dir.join("示例 #a #b.txt");
        fs::write(&path, "content").unwrap();
        let meta = TagsMeta::default();
        let prompt = read_prompt(&path, &dir, &meta, 50).expect("read prompt");

        assert!(tags_match(&prompt, &["a".to_string()]));
        assert!(tags_match(&prompt, &["a".to_string(), "b".to_string()]));
        assert!(!tags_match(&prompt, &["a".to_string(), "c".to_string()]));
    }

    #[test]
    fn best_substring_index_handles_multibyte() {
        assert_eq!(best_substring_index("中文测试", "文"), Some(3));
    }

    #[test]
    fn score_match_handles_chinese() {
        assert!(score_match("中文测试", "测").is_some());
    }
}
