use super::{compare_dirs_first, natural_cmp, FileSorter, SortOptions};
use crate::error::Result;
use rayon::prelude::*;
use std::path::PathBuf;

pub struct AlphabeticalSorter;

impl FileSorter for AlphabeticalSorter {
    fn sort_files(&self, files: &mut [PathBuf], options: SortOptions) -> Result<()> {
        files.par_sort_unstable_by(|a, b| {
            let dir_order = compare_dirs_first(a, b, options.dirs_first);
            if dir_order != std::cmp::Ordering::Equal {
                return if options.reverse {
                    dir_order.reverse()
                } else {
                    dir_order
                };
            }

            let a_name = a.file_name().unwrap_or_default().to_string_lossy();
            let b_name = b.file_name().unwrap_or_default().to_string_lossy();

            let name_order = if options.natural {
                natural_cmp(&a_name, &b_name)
            } else if options.case_sensitive {
                a_name.cmp(&b_name)
            } else {
                a_name.to_lowercase().cmp(&b_name.to_lowercase())
            };

            if options.reverse {
                name_order.reverse()
            } else {
                name_order
            }
        });

        Ok(())
    }
}
