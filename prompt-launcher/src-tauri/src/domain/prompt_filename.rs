use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptFileNameError {
    Empty,
    Invalid,
}

impl PromptFileNameError {
    pub fn message(&self) -> &'static str {
        match self {
            PromptFileNameError::Empty => "文件名不能为空",
            PromptFileNameError::Invalid => "文件名包含非法字符",
        }
    }
}

impl fmt::Display for PromptFileNameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message())
    }
}

impl std::error::Error for PromptFileNameError {}

pub fn build_prompt_file_name(input: &str) -> Result<String, PromptFileNameError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(PromptFileNameError::Empty);
    }
    let file_name = if trimmed.to_ascii_lowercase().ends_with(".txt") {
        trimmed.to_string()
    } else {
        format!("{trimmed}.txt")
    };
    if !is_valid_filename(&file_name) {
        return Err(PromptFileNameError::Invalid);
    }
    Ok(file_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_prompt_file_name_rejects_empty() {
        // Given: an empty file name input
        // When: building a prompt file name
        // Then: it should reject the input as empty
        assert_eq!(build_prompt_file_name(" "), Err(PromptFileNameError::Empty));
    }

    #[test]
    fn build_prompt_file_name_rejects_invalid() {
        // Given: an input containing invalid filename characters
        // When: building a prompt file name
        // Then: it should reject the input as invalid
        assert_eq!(
            build_prompt_file_name("bad|name"),
            Err(PromptFileNameError::Invalid)
        );
    }

    #[test]
    fn build_prompt_file_name_appends_txt() {
        // Given: a valid name without a .txt extension
        // When: building a prompt file name
        // Then: it should append .txt
        assert_eq!(build_prompt_file_name("hello"), Ok("hello.txt".to_string()));
    }

    #[test]
    fn build_prompt_file_name_keeps_txt() {
        // Given: a valid name that already ends with .txt
        // When: building a prompt file name
        // Then: it should keep the original extension
        assert_eq!(
            build_prompt_file_name("demo.txt"),
            Ok("demo.txt".to_string())
        );
    }
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
    !trimmed.chars().any(|ch| invalid.contains(&ch))
}
