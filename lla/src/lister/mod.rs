use crate::error::Result;
use std::path::PathBuf;

pub trait FileLister {
    fn list_files(
        &self,
        directory: &str,
        recursive: bool,
        depth: Option<usize>,
    ) -> Result<Vec<PathBuf>>;
}

mod basic;
mod recursive;

pub use basic::BasicLister;
pub use recursive::RecursiveLister;
