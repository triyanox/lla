use std::fs::Metadata;
use std::path::PathBuf;

pub struct AlphabeticalSorter;
pub struct SizeSorter;
pub struct DateSorter;

pub trait FileSorter {
    fn sort_files(&self, files: Vec<PathBuf>) -> Vec<PathBuf>;
}

impl AlphabeticalSorter {
    pub fn new() -> AlphabeticalSorter {
        AlphabeticalSorter {}
    }
}

impl FileSorter for AlphabeticalSorter {
    fn sort_files(&self, mut files: Vec<PathBuf>) -> Vec<PathBuf> {
        files.sort();
        files
    }
}

impl SizeSorter {
    pub fn new() -> SizeSorter {
        SizeSorter {}
    }

    fn get_file_size(&self, file: &PathBuf) -> Option<u64> {
        if let Ok(metadata) = file.metadata() {
            Some(metadata.len())
        } else {
            None
        }
    }
}

impl FileSorter for SizeSorter {
    fn sort_files(&self, mut files: Vec<PathBuf>) -> Vec<PathBuf> {
        files.sort_by(|a, b| {
            let size_a = self.get_file_size(a).unwrap_or(u64::MAX);
            let size_b = self.get_file_size(b).unwrap_or(u64::MAX);
            size_a.cmp(&size_b)
        });
        files
    }
}

impl DateSorter {
    pub fn new() -> DateSorter {
        DateSorter {}
    }

    fn get_modification_date(&self, file: &PathBuf) -> Option<Metadata> {
        if let Ok(metadata) = file.metadata() {
            Some(metadata)
        } else {
            None
        }
    }
}

impl FileSorter for DateSorter {
    fn sort_files(&self, mut files: Vec<PathBuf>) -> Vec<PathBuf> {
        files.sort_by(|a, b| {
            let date_a = self.get_modification_date(a).unwrap_or_else(|| {
                panic!("Failed to get modification date for file {:?}", a);
            });
            let date_b = self.get_modification_date(b).unwrap_or_else(|| {
                panic!("Failed to get modification date for file {:?}", b);
            });
            date_a.modified().unwrap().cmp(&date_b.modified().unwrap())
        });
        files
    }
}
