use super::FileLister;
use crate::error::Result;
use crate::lister::BasicLister;
use std::fs;
use std::path::{Path, PathBuf};

pub struct RecursiveLister;

impl FileLister for RecursiveLister {
    fn list_files(
        &self,
        directory: &str,
        recursive: bool,
        depth: Option<usize>,
    ) -> Result<Vec<PathBuf>> {
        fn list_recursive(
            dir: &Path,
            max_depth: Option<usize>,
            current_depth: usize,
        ) -> Result<Vec<PathBuf>> {
            let mut files = Vec::new();
            if max_depth.map_or(true, |d| current_depth < d) {
                for entry in fs::read_dir(dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    files.push(path.clone());
                    if path.is_dir() {
                        files.extend(list_recursive(&path, max_depth, current_depth + 1)?);
                    }
                }
            }
            Ok(files)
        }

        if recursive {
            list_recursive(Path::new(directory), depth, 0)
        } else {
            BasicLister.list_files(directory, false, None)
        }
    }
}
