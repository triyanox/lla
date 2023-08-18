use crate::error::LlaError;
use crate::ls::LongLister;
use std::process::exit;
use std::{fs::Metadata, os::unix::prelude::MetadataExt, path::PathBuf};

pub fn get_file_type(metadata: &Metadata) -> String {
    let file_type = if metadata.is_file() {
        "file"
    } else if metadata.is_dir() {
        "dir"
    } else if metadata.file_type().is_symlink() {
        "link"
    } else {
        "other"
    };

    file_type.to_string()
}

pub struct LongFile {
    pub file_type: String,
    pub permissions: String,
    pub links: u64,
    pub user: String,
    pub group: String,
    pub size: u64,
    pub date: chrono::DateTime<chrono::Local>,
    pub name: String,
}

impl LongFile {
    pub fn new(file: &PathBuf) -> LongFile {
        let metadata = file
            .metadata()
            .map_err(|e| LlaError::FailedToGetMetadata(e.to_string()))
            .unwrap();
        let file_type = get_file_type(&metadata);
        let permissions = LongLister::new().get_file_permissions(&metadata);
        let links = metadata.nlink();
        let size = metadata.len();
        let date = chrono::DateTime::from(metadata.modified().unwrap());
        let name = file.file_name().unwrap().to_string_lossy().to_string();

        let user: String;
        let group: String;

        let u = LongLister::new().get_user_name(&metadata);
        if let Err(e) = u {
            eprintln!("{}", e);
            exit(1);
        } else {
            user = u.unwrap();
        }

        let g = LongLister::new().get_group_name(&metadata);
        if let Err(e) = g {
            eprintln!("{}", e);
            exit(1);
        } else {
            group = g.unwrap();
        }

        LongFile {
            file_type,
            permissions,
            links,
            user,
            group,
            size,
            date,
            name,
        }
    }
}
