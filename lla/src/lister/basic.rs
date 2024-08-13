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
        let entries = fs::read_dir(directory)?;
        let mut files = Vec::new();

        for entry in entries {
            let entry = entry?;
            files.push(entry.path());
        }

        Ok(files)
    }
}
