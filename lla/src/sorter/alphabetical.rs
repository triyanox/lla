use super::FileSorter;
use crate::error::Result;
use std::path::PathBuf;
use rayon::prelude::*;

pub struct AlphabeticalSorter;

impl FileSorter for AlphabeticalSorter {
    fn sort_files(&self, files: &mut [PathBuf]) -> Result<()> {
        files.par_sort_unstable_by_key(|a| a.file_name().unwrap_or_default().to_owned());
        Ok(())
    }
}
