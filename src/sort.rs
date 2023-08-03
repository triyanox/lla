use crate::error::LlaError;
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
            let date_a = self
                .get_modification_date(a)
                .ok_or_else(|| LlaError::FailedToGetMetadata(a.to_string_lossy().to_string()));
            let date_b = self
                .get_modification_date(b)
                .ok_or_else(|| LlaError::FailedToGetMetadata(b.to_string_lossy().to_string()));
            date_a
                .map_err(|e| eprintln!("{}", e))
                .unwrap_or_else(|_| std::process::exit(1))
                .modified()
                .map_err(|e| eprintln!("{}", e))
                .unwrap_or_else(|_| std::process::exit(1))
                .cmp(&date_b.unwrap().modified().unwrap())
        });

        files
    }
}
