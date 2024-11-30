use super::FileSorter;
use crate::error::Result;
use rayon::prelude::*;
use std::path::PathBuf;
use std::time::SystemTime;

pub struct DateSorter;

impl FileSorter for DateSorter {
    fn sort_files(&self, files: &mut [PathBuf]) -> Result<()> {
        let times: Vec<_> = files
            .par_iter()
            .map(|path| {
                path.metadata()
                    .and_then(|m| m.modified())
                    .unwrap_or(SystemTime::UNIX_EPOCH)
            })
            .collect();

        let mut indices: Vec<usize> = (0..files.len()).collect();

        indices.par_sort_unstable_by_key(|&i| std::cmp::Reverse(times[i]));

        let temp = files.to_vec();
        for (i, &idx) in indices.iter().enumerate() {
            files[i] = temp[idx].clone();
        }

        Ok(())
    }
}
