use super::{compare_dirs_first, natural_cmp, FileSorter, SortOptions};
use crate::error::Result;
use lla_plugin_interface::proto::DecoratedEntry;
use rayon::prelude::*;
use std::path::PathBuf;

pub struct AlphabeticalSorter;

impl FileSorter for AlphabeticalSorter {
    fn sort_files_with_metadata(
        &self,
        entries: &mut [(PathBuf, &DecoratedEntry)],
        options: SortOptions,
    ) -> Result<()> {
        entries.par_sort_unstable_by(|(path_a, _), (path_b, _)| {
            let dir_order = compare_dirs_first(path_a, path_b, options.dirs_first);
            if dir_order != std::cmp::Ordering::Equal {
                return if options.reverse {
                    dir_order.reverse()
                } else {
                    dir_order
                };
            }

            let a_name = path_a.file_name().unwrap_or_default().to_string_lossy();
            let b_name = path_b.file_name().unwrap_or_default().to_string_lossy();

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
