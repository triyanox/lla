use super::FileLister;
use crate::error::Result;
use std::fs;
use std::path::PathBuf;

pub struct BasicLister;

impl FileLister for BasicLister {
    fn list_files(
        &self,
        directory: &str,
        _recursive: bool,
        _depth: Option<usize>,
    ) -> Result<Vec<PathBuf>> {
        let mut files = Vec::with_capacity(16);

        let entries = fs::read_dir(directory)?;
        for entry in entries {
            if let Ok(entry) = entry {
                files.push(entry.path());
            }
        }

        Ok(files)
    }
}
