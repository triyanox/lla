use super::FileSorter;
use crate::error::Result;
use std::path::PathBuf;

pub struct AlphabeticalSorter;

impl FileSorter for AlphabeticalSorter {
    fn sort_files(&self, files: &mut [PathBuf]) -> Result<()> {
        files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
        Ok(())
    }
}
