use super::FileFilter;
use crate::error::Result;
use glob::Pattern;
use std::path::PathBuf;

pub struct GlobFilter {
    pattern: Pattern,
}

impl GlobFilter {
    pub fn new(pattern: String) -> Self {
        GlobFilter {
            pattern: Pattern::new(&pattern).unwrap_or_else(|_| Pattern::new("*").unwrap()),
        }
    }
}

impl FileFilter for GlobFilter {
    fn filter_files(&self, files: &[PathBuf]) -> Result<Vec<PathBuf>> {
        Ok(files
            .iter()
            .filter(|file| {
                file.file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| self.pattern.matches(name))
                    .unwrap_or(false)
            })
            .cloned()
            .collect())
    }
}
