use std::path::PathBuf;
use std::vec::Vec;

pub trait FileFilter {
    fn filter_files(&self, files: Vec<PathBuf>) -> Vec<PathBuf>;
}

pub struct ExtensionFilter {
    extension: String,
}

impl ExtensionFilter {
    pub fn new(extension: &str) -> ExtensionFilter {
        ExtensionFilter {
            extension: extension.to_string(),
        }
    }
}

impl FileFilter for ExtensionFilter {
    fn filter_files(&self, files: Vec<PathBuf>) -> Vec<PathBuf> {
        let extension = self.extension.trim_start_matches('.').to_lowercase();
        files
            .into_iter()
            .filter(|file| {
                file.extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext_str| ext_str.to_lowercase() == extension)
                    .unwrap_or(false)
            })
            .collect()
    }
}
pub struct PatternFilter {
    pattern: String,
}

impl PatternFilter {
    pub fn new(pattern: &str) -> PatternFilter {
        PatternFilter {
            pattern: pattern.to_string(),
        }
    }
}

impl FileFilter for PatternFilter {
    fn filter_files(&self, files: Vec<PathBuf>) -> Vec<PathBuf> {
        let pattern = self.pattern.to_lowercase();
        files
            .into_iter()
            .filter(|file| {
                file.file_name()
                    .and_then(|name| name.to_str())
                    .map(|name_str| name_str.to_lowercase().contains(&pattern))
                    .unwrap_or(false)
            })
            .collect()
    }
}

pub struct NoFilter {}

impl NoFilter {
    pub fn new() -> NoFilter {
        NoFilter {}
    }
}

impl FileFilter for NoFilter {
    fn filter_files(&self, files: Vec<PathBuf>) -> Vec<PathBuf> {
        files
    }
}
