use super::FileFilter;
use crate::error::Result;
use std::path::PathBuf;

pub struct PatternFilter {
    pattern: String,
}

impl PatternFilter {
    pub fn new(pattern: String) -> Self {
        PatternFilter { pattern }
    }
}

impl FileFilter for PatternFilter {
    fn filter_files(&self, files: &[PathBuf]) -> Result<Vec<PathBuf>> {
        Ok(files
            .iter()
            .filter(|file| {
                file.file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| name.contains(&self.pattern))
                    .unwrap_or(false)
            })
            .cloned()
            .collect())
    }
}
