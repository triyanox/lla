use super::{compare_dirs_first, FileSorter, SortOptions};
use crate::error::Result;
use rayon::prelude::*;
use std::path::PathBuf;

pub struct SizeSorter;

impl FileSorter for SizeSorter {
    fn sort_files(&self, files: &mut [PathBuf], options: SortOptions) -> Result<()> {
        let sizes: Vec<_> = files
            .par_iter()
            .map(|path| path.metadata().map(|m| m.len()).unwrap_or(0))
            .collect();

        let mut indices: Vec<usize> = (0..files.len()).collect();
        indices.par_sort_unstable_by(|&i, &j| {
            let dir_order = compare_dirs_first(&files[i], &files[j], options.dirs_first);
            if dir_order != std::cmp::Ordering::Equal {
                return if options.reverse {
                    dir_order.reverse()
                } else {
                    dir_order
                };
            }

            let size_order = sizes[i].cmp(&sizes[j]);
            if options.reverse {
                size_order
            } else {
                size_order.reverse()
            }
        });

        let temp = files.to_vec();
        for (i, &idx) in indices.iter().enumerate() {
            files[i] = temp[idx].clone();
        }

        Ok(())
    }
}
