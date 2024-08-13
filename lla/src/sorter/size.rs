use super::FileSorter;
use crate::error::Result;
use std::path::PathBuf;

pub struct SizeSorter;

impl FileSorter for SizeSorter {
    fn sort_files(&self, files: &mut [PathBuf]) -> Result<()> {
        files.sort_by(|a, b| {
            let size_a = a.metadata().map(|m| m.len()).unwrap_or(0);
            let size_b = b.metadata().map(|m| m.len()).unwrap_or(0);
            size_b.cmp(&size_a)
        });
        Ok(())
    }
}
