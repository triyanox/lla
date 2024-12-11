use super::FileFilter;
use crate::error::Result;
use regex::Regex;
use std::path::PathBuf;

pub struct RegexFilter {
    regex: Regex,
}

impl RegexFilter {
    pub fn new(pattern: String) -> Self {
        RegexFilter {
            regex: Regex::new(&pattern).unwrap_or_else(|_| Regex::new(".*").unwrap()),
        }
    }
}

impl FileFilter for RegexFilter {
    fn filter_files(&self, files: &[PathBuf]) -> Result<Vec<PathBuf>> {
        Ok(files
            .iter()
            .filter(|file| {
                file.file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| self.regex.is_match(name))
                    .unwrap_or(false)
            })
            .cloned()
            .collect())
    }
}
