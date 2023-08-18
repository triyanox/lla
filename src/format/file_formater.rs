use std::path::PathBuf;

pub trait FileFormatter {
    fn display_files(&self, files: &[PathBuf], long: Option<bool>, git: Option<bool>);
}
