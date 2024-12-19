use super::{compare_dirs_first, FileSorter, SortOptions};
use crate::error::Result;
use lla_plugin_interface::proto::DecoratedEntry;
use rayon::prelude::*;
use std::path::PathBuf;

pub struct SizeSorter;

impl FileSorter for SizeSorter {
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

            let size_a = entry_a.metadata.as_ref().map_or(0, |m| m.size);
            let size_b = entry_b.metadata.as_ref().map_or(0, |m| m.size);
            let size_order = size_a.cmp(&size_b);

            if options.reverse {
                size_order
            } else {
                size_order.reverse()
            }
        });

        Ok(())
    }
}
