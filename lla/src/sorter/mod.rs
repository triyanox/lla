use crate::error::Result;
use std::path::PathBuf;

#[derive(Clone, Copy, Default)]
pub struct SortOptions {
    pub reverse: bool,
    pub dirs_first: bool,
    pub case_sensitive: bool,
    pub natural: bool,
}

pub trait FileSorter: Send + Sync {
    fn sort_files(&self, files: &mut [PathBuf], options: SortOptions) -> Result<()>;
}

mod alphabetical;
mod date;
mod size;

pub use alphabetical::AlphabeticalSorter;
pub use date::DateSorter;
pub use size::SizeSorter;

pub(crate) fn compare_dirs_first(a: &PathBuf, b: &PathBuf, dirs_first: bool) -> std::cmp::Ordering {
    if !dirs_first {
        return std::cmp::Ordering::Equal;
    }

    let a_is_dir = a.is_dir();
    let b_is_dir = b.is_dir();

    match (a_is_dir, b_is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => std::cmp::Ordering::Equal,
    }
}

pub(crate) fn natural_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let mut a_parts = a.split_inclusive(char::is_numeric);
    let mut b_parts = b.split_inclusive(char::is_numeric);

    loop {
        match (a_parts.next(), b_parts.next()) {
            (None, None) => return std::cmp::Ordering::Equal,
            (None, Some(_)) => return std::cmp::Ordering::Less,
            (Some(_), None) => return std::cmp::Ordering::Greater,
            (Some(a_part), Some(b_part)) => {
                if a_part.ends_with(char::is_numeric) && b_part.ends_with(char::is_numeric) {
                    let a_num: u64 = a_part
                        .trim_end_matches(char::is_numeric)
                        .parse()
                        .unwrap_or(0);
                    let b_num: u64 = b_part
                        .trim_end_matches(char::is_numeric)
                        .parse()
                        .unwrap_or(0);
                    match a_num.cmp(&b_num) {
                        std::cmp::Ordering::Equal => continue,
                        other => return other,
                    }
                } else {
                    match a_part.cmp(b_part) {
                        std::cmp::Ordering::Equal => continue,
                        other => return other,
                    }
                }
            }
        }
    }
}
