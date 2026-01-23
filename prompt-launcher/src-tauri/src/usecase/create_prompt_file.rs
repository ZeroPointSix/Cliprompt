use std::path::{Path, PathBuf};

use crate::domain::prompt_filename::build_prompt_file_name;

pub trait PromptFileRepository {
    fn ensure_dir(&self, dir: &Path) -> Result<(), String>;
    fn exists(&self, path: &Path) -> bool;
    fn create_new(&self, path: &Path) -> Result<(), String>;
}

pub struct CreatePromptFileUseCase<R> {
    repo: R,
}

impl<R> CreatePromptFileUseCase<R>
where
    R: PromptFileRepository,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub fn execute(&self, prompts_dir: &Path, name: &str) -> Result<PathBuf, String> {
        let file_name = build_prompt_file_name(name).map_err(|err| err.to_string())?;
        self.repo.ensure_dir(prompts_dir)?;
        let path = prompts_dir.join(file_name);
        if self.repo.exists(&path) {
            return Err("文件已存在，无法创建".to_string());
        }
        self.repo.create_new(&path)?;
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashSet;

    #[derive(Default)]
    struct MemoryRepo {
        ensured_dirs: RefCell<Vec<PathBuf>>,
        existing: RefCell<HashSet<PathBuf>>,
        created: RefCell<Vec<PathBuf>>,
    }

    impl PromptFileRepository for MemoryRepo {
        fn ensure_dir(&self, dir: &Path) -> Result<(), String> {
            self.ensured_dirs.borrow_mut().push(dir.to_path_buf());
            Ok(())
        }

        fn exists(&self, path: &Path) -> bool {
            self.existing.borrow().contains(path)
        }

        fn create_new(&self, path: &Path) -> Result<(), String> {
            self.created.borrow_mut().push(path.to_path_buf());
            Ok(())
        }
    }

    #[test]
    fn execute_rejects_empty_name() {
        // Given: an empty name and a prompt directory
        // When: executing the use case
        // Then: it should reject the request with a "name empty" error
        let repo = MemoryRepo::default();
        let usecase = CreatePromptFileUseCase::new(repo);
        let result = usecase.execute(Path::new("C:/prompts"), " ");
        assert_eq!(result, Err("文件名不能为空".to_string()));
    }

    #[test]
    fn execute_rejects_invalid_name() {
        // Given: a name that includes invalid characters
        // When: executing the use case
        // Then: it should reject the request with an "invalid name" error
        let repo = MemoryRepo::default();
        let usecase = CreatePromptFileUseCase::new(repo);
        let result = usecase.execute(Path::new("C:/prompts"), "bad:name");
        assert_eq!(result, Err("文件名包含非法字符".to_string()));
    }

    #[test]
    fn execute_appends_txt_extension() {
        // Given: a valid name without an extension
        // When: executing the use case
        // Then: it should append .txt
        let repo = MemoryRepo::default();
        let usecase = CreatePromptFileUseCase::new(repo);
        let result = usecase
            .execute(Path::new("C:/prompts"), "hello")
            .expect("should create");
        assert_eq!(result, Path::new("C:/prompts/hello.txt"));
    }

    #[test]
    fn execute_rejects_existing_file() {
        // Given: a prompt file that already exists
        // When: executing the use case
        // Then: it should reject the request with an "exists" error
        let repo = MemoryRepo::default();
        repo.existing
            .borrow_mut()
            .insert(PathBuf::from("C:/prompts/existing.txt"));
        let usecase = CreatePromptFileUseCase::new(repo);
        let result = usecase.execute(Path::new("C:/prompts"), "existing.txt");
        assert_eq!(result, Err("文件已存在，无法创建".to_string()));
    }

    #[test]
    fn execute_creates_file_once() {
        // Given: a new prompt name in a valid directory
        // When: executing the use case
        // Then: it should return the expected prompt path
        let repo = MemoryRepo::default();
        let usecase = CreatePromptFileUseCase::new(repo);
        let result = usecase
            .execute(Path::new("C:/prompts"), "demo")
            .expect("should create");
        assert_eq!(result, Path::new("C:/prompts/demo.txt"));
    }
}
