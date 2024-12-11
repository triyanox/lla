use super::{compare_dirs_first, FileSorter, SortOptions};
use crate::error::Result;
use rayon::prelude::*;
use std::path::PathBuf;
use std::time::SystemTime;

pub struct DateSorter;

impl FileSorter for DateSorter {
    fn sort_files(&self, files: &mut [PathBuf], options: SortOptions) -> Result<()> {
        let times: Vec<_> = files
            .par_iter()
            .map(|path| {
                path.metadata()
                    .and_then(|m| m.modified())
                    .unwrap_or(SystemTime::UNIX_EPOCH)
            })
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

            let date_order = times[i].cmp(&times[j]);
            if options.reverse {
                date_order.reverse()
            } else {
                date_order
            }
        });

        let temp = files.to_vec();
        for (i, &idx) in indices.iter().enumerate() {
            files[i] = temp[idx].clone();
        }

        Ok(())
    }
}
