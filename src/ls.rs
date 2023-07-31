use std::fs::{self, Metadata};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::prelude::MetadataExt;
use std::path::PathBuf;
use std::vec::Vec;

pub trait FileLister {
    fn list_files(
        &self,
        directory: &str,
        recursive: Option<bool>,
        depth: Option<u32>,
    ) -> Vec<PathBuf>;
}

pub struct BasicLister;

impl BasicLister {
    pub fn new() -> BasicLister {
        BasicLister {}
    }
}

impl FileLister for BasicLister {
    fn list_files(
        &self,
        directory: &str,
        _recursive: Option<bool>,
        _depth: Option<u32>,
    ) -> Vec<PathBuf> {
        let entries = fs::read_dir(directory).unwrap();
        let mut files = Vec::new();

        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                files.push(path);
            }
        }

        files
    }
}

pub struct LongLister;

impl LongLister {
    pub fn new() -> LongLister {
        LongLister {}
    }

    pub fn get_file_permissions(&self, metadata: &Metadata) -> String {
        let mode = metadata.permissions().mode();
        let mut permissions = String::new();

        let modes_and_permissions = [
            (0o100000, "-"),
            (0o120000, "l"),
            (0o040000, "d"),
            (0o100000, "p"),
            (0o060000, "b"),
            (0o060000, "c"),
            (0o140000, "s"),
        ];
        for &(m, p) in modes_and_permissions.iter() {
            if mode & m == m {
                permissions.push_str(p);
            } else {
                permissions.push_str("-");
            }
        }
        permissions.push_str(&self.get_file_mode(mode));
        permissions
    }

    pub fn get_file_mode(&self, mode: u32) -> String {
        let mut file_mode = String::new();

        let modes_and_permissions = [
            (0o400, "r"),
            (0o200, "w"),
            (0o100, "x"),
            (0o040, "r"),
            (0o020, "w"),
            (0o010, "x"),
            (0o004, "r"),
            (0o002, "w"),
            (0o001, "x"),
        ];
        for &(m, p) in modes_and_permissions.iter() {
            if mode & m == m {
                file_mode.push_str(p);
            } else {
                file_mode.push_str("-");
            }
        }
        file_mode
    }

    pub fn get_user_name(&self, metadata: &Metadata) -> String {
        let uid = metadata.uid();
        let user = users::get_user_by_uid(uid).unwrap();
        user.name().to_string_lossy().to_string()
    }

    pub fn get_group_name(&self, metadata: &Metadata) -> String {
        let gid = metadata.gid();
        let group = users::get_group_by_gid(gid).unwrap();
        group.name().to_string_lossy().to_string()
    }

    pub fn format_size(&self, size: u64) -> String {
        let sizes = ["B", "KB", "MB", "GB", "TB"];
        let mut size = size as f64;
        let mut idx = 0;

        while size >= 1024.0 && idx < sizes.len() - 1 {
            size /= 1024.0;
            idx += 1;
        }

        format!("{:.1} {}", size, sizes[idx])
    }
}

impl FileLister for LongLister {
    fn list_files(
        &self,
        directory: &str,
        _recursive: Option<bool>,
        _depth: Option<u32>,
    ) -> Vec<PathBuf> {
        let entries = fs::read_dir(directory).unwrap();
        let mut files = Vec::new();

        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                files.push(path);
            }
        }

        files
    }
}

pub struct TreeLister;

impl TreeLister {
    pub fn new() -> TreeLister {
        TreeLister {}
    }
}

impl FileLister for TreeLister {
    fn list_files(
        &self,
        directory: &str,
        recursive: Option<bool>,
        depth: Option<u32>,
    ) -> Vec<PathBuf> {
        if (depth.is_some() && depth.unwrap() == 0) || (recursive.is_some() && !recursive.unwrap())
        {
            return Vec::new();
        } else {
            let entries = fs::read_dir(directory).unwrap();
            let mut files = Vec::new();

            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_dir() {
                        let mut new_depth = depth.unwrap_or(0);
                        if depth.is_some() {
                            new_depth -= 1;
                        }
                        let mut new_recursive = recursive.unwrap_or(false);
                        if recursive.is_some() {
                            new_recursive = recursive.unwrap();
                        }
                        let mut new_directory = directory.to_string();
                        new_directory.push_str("/");
                        new_directory.push_str(path.file_name().unwrap().to_str().unwrap());
                        files.push(path);
                        files.append(&mut self.list_files(
                            &new_directory,
                            Some(new_recursive),
                            Some(new_depth),
                        ));
                    } else {
                        files.push(path);
                    }
                }
            }

            files
        }
    }
}
