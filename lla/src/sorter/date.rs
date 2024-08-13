use super::FileSorter;
use crate::error::Result;
use std::path::PathBuf;

pub struct DateSorter;

impl FileSorter for DateSorter {
    fn sort_files(&self, files: &mut [PathBuf]) -> Result<()> {
        files.sort_by(|a, b| {
            let time_a = a
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or_else(|_| std::time::SystemTime::UNIX_EPOCH);
            let time_b = b
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or_else(|_| std::time::SystemTime::UNIX_EPOCH);
            time_b.cmp(&time_a)
        });
        Ok(())
    }
}
