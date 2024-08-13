use crate::error::Result;
use std::path::PathBuf;

pub trait FileSorter {
    fn sort_files(&self, files: &mut [PathBuf]) -> Result<()>;
}

mod alphabetical;
mod date;
mod size;

pub use alphabetical::AlphabeticalSorter;
pub use date::DateSorter;
pub use size::SizeSorter;
