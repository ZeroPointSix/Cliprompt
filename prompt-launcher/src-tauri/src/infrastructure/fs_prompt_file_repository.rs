use std::fs;
use std::path::Path;

use crate::usecase::create_prompt_file::PromptFileRepository;

pub struct FsPromptFileRepository;

impl PromptFileRepository for FsPromptFileRepository {
    fn ensure_dir(&self, dir: &Path) -> Result<(), String> {
        fs::create_dir_all(dir).map_err(|e| format!("create prompts dir failed: {e}"))
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn create_new(&self, path: &Path) -> Result<(), String> {
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map(|_| ())
            .map_err(|e| format!("create file failed: {e}"))
    }
}
