use super::FileSorter;
use crate::error::Result;
use rayon::prelude::*;
use std::path::PathBuf;

pub struct SizeSorter;

impl FileSorter for SizeSorter {
    fn sort_files(&self, files: &mut [PathBuf]) -> Result<()> {
        let sizes: Vec<_> = files
            .par_iter()
            .map(|path| path.metadata().map(|m| m.len()).unwrap_or(0))
            .collect();
        let mut indices: Vec<usize> = (0..files.len()).collect();
        indices.par_sort_unstable_by_key(|&i| std::cmp::Reverse(sizes[i]));
        let temp = files.to_vec();
        for (i, &idx) in indices.iter().enumerate() {
            files[i] = temp[idx].clone();
        }
        Ok(())
    }
}
