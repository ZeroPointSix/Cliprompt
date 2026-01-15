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

fn split_query(query: &str) -> (Vec<String>, Vec<String>) {
    let mut tags = Vec::new();
    let mut terms = Vec::new();

    for token in query.split_whitespace() {
        if let Some(tag) = token.strip_prefix('#') {
            if let Some(normalized) = normalize_tag(tag) {
                tags.push(normalized);
            }
        } else {
            terms.push(token);
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
    if let Some(score) = score_terms(&title_text, terms) {
        best = best.min(score - 120);
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
    let mut best: Option<usize> = None;
    let mut start = 0usize;
    while let Some(pos) = text[start..].find(term) {
        let idx = start + pos;
        best = Some(match best {
            Some(current) => {
                if idx < current {
                    idx
                } else {
                    current
                }
            }
            None => idx,
        });
        start = idx + 1;
    }
    best
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
