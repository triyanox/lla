use super::FileFilter;
use crate::error::Result;
use std::path::PathBuf;

pub struct ExtensionFilter {
    extension: String,
}

impl ExtensionFilter {
    pub fn new(extension: String) -> Self {
        ExtensionFilter { extension }
    }
}

impl FileFilter for ExtensionFilter {
    fn filter_files(&self, files: &[PathBuf]) -> Result<Vec<PathBuf>> {
        Ok(files
            .iter()
            .filter(|file| {
                file.extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext == self.extension)
                    .unwrap_or(false)
            })
            .cloned()
            .collect())
    }
}
