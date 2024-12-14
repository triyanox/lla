use super::FileFilter;
use crate::error::Result;
use std::path::PathBuf;

pub struct CaseInsensitiveFilter {
    inner: Box<dyn FileFilter>,
}

impl CaseInsensitiveFilter {
    pub fn new(inner: Box<dyn FileFilter>) -> Self {
        CaseInsensitiveFilter { inner }
    }

    fn to_lowercase_path(path: &PathBuf) -> PathBuf {
        let parent = path.parent().unwrap_or_else(|| path.as_ref());
        let filename = path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.to_lowercase())
            .unwrap_or_default();
        parent.join(filename)
    }
}

impl FileFilter for CaseInsensitiveFilter {
    fn filter_files(&self, files: &[PathBuf]) -> Result<Vec<PathBuf>> {
        let lowercase_files: Vec<PathBuf> = files.iter().map(Self::to_lowercase_path).collect();

        let filtered = self.inner.filter_files(&lowercase_files)?;
        let filtered_lowercase: Vec<PathBuf> =
            filtered.iter().map(Self::to_lowercase_path).collect();

        Ok(files
            .iter()
            .enumerate()
            .filter(|(i, _)| {
                let lowercase_path = Self::to_lowercase_path(&lowercase_files[*i]);
                filtered_lowercase.contains(&lowercase_path)
            })
            .map(|(_, path)| path.clone())
            .collect())
    }
}
