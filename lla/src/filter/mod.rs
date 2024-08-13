use crate::error::Result;
use std::path::PathBuf;

pub trait FileFilter {
    fn filter_files(&self, files: &[PathBuf]) -> Result<Vec<PathBuf>>;
}

mod extension;
mod pattern;

pub use extension::ExtensionFilter;
pub use pattern::PatternFilter;
