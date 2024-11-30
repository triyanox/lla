use super::FileLister;
use crate::error::Result;
use crate::lister::BasicLister;
use rayon::prelude::*;
use std::path::PathBuf;
use walkdir::WalkDir;

pub struct RecursiveLister;

impl FileLister for RecursiveLister {
    fn list_files(
        &self,
        directory: &str,
        recursive: bool,
        depth: Option<usize>,
    ) -> Result<Vec<PathBuf>> {
        if !recursive {
            return BasicLister.list_files(directory, false, None);
        }

        let max_depth = depth.unwrap_or(usize::MAX);
        let mut entries = Vec::with_capacity(128);
        let walker = WalkDir::new(directory)
            .min_depth(0)
            .max_depth(max_depth)
            .follow_links(false)
            .same_file_system(true);

        for entry in walker.into_iter().filter_map(|e| e.ok()) {
            entries.push(entry.into_path());
        }

        entries.par_sort_unstable();
        Ok(entries)
    }
}
