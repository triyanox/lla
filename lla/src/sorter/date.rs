use super::{compare_dirs_first, FileSorter, SortOptions};
use crate::error::Result;
use lla_plugin_interface::proto::DecoratedEntry;
use rayon::prelude::*;
use std::path::PathBuf;

pub struct DateSorter;

impl FileSorter for DateSorter {
    fn sort_files_with_metadata(
        &self,
        entries: &mut [(PathBuf, &DecoratedEntry)],
        options: SortOptions,
    ) -> Result<()> {
        entries.par_sort_unstable_by(|(path_a, entry_a), (path_b, entry_b)| {
            let dir_order = compare_dirs_first(path_a, path_b, options.dirs_first);
            if dir_order != std::cmp::Ordering::Equal {
                return if options.reverse {
                    dir_order.reverse()
                } else {
                    dir_order
                };
            }

            let time_a = entry_a.metadata.as_ref().map_or(0, |m| m.modified);
            let time_b = entry_b.metadata.as_ref().map_or(0, |m| m.modified);
            let date_order = time_a.cmp(&time_b);

            if options.reverse {
                date_order.reverse()
            } else {
                date_order
            }
        });

        Ok(())
    }
}
