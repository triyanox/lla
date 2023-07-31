use crate::ls::LongLister;
use std::collections::HashMap;
use std::{fs::Metadata, os::unix::prelude::MetadataExt, path::PathBuf};
use termion::{color, style};

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

pub struct DefaultFileFormatter;
pub struct TreeFileFormatter;
pub trait FileFormatter {
    fn display_files(&self, files: &[PathBuf], long: Option<bool>);
}

impl LongFile {
    pub fn new(file: &PathBuf) -> LongFile {
        let metadata = file.metadata().expect("Failed to get file metadata");
        let file_type = get_file_type(&metadata);
        let permissions = LongLister::new().get_file_permissions(&metadata);
        let links = metadata.nlink();
        let user = LongLister::new().get_user_name(&metadata);
        let group = LongLister::new().get_group_name(&metadata);
        let size = metadata.len();
        let date = chrono::DateTime::from(metadata.modified().unwrap());
        let name = file.file_name().unwrap().to_string_lossy().to_string();

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

fn get_file_type(metadata: &Metadata) -> String {
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

impl DefaultFileFormatter {
    pub fn new() -> DefaultFileFormatter {
        DefaultFileFormatter
    }
}

impl FileFormatter for DefaultFileFormatter {
    fn display_files(&self, files: &[PathBuf], long: Option<bool>) {
        if let Some(true) = long {
            self.display_files_long_format(files);
        } else {
            self.display_files_default_format(files);
        }
    }
}

impl DefaultFileFormatter {
    fn display_files_default_format(&self, files: &[PathBuf]) {
        for file in files {
            if let Some(name) = file.file_name() {
                if let Some(name_str) = name.to_str() {
                    if file.is_dir() {
                        println!("{}{}{}", color::Fg(color::Blue), name_str, style::Reset);
                    } else {
                        println!("{}", name_str);
                    }
                }
            }
        }
    }

    fn display_files_long_format(&self, files: &[PathBuf]) {
        if files.is_empty() {
            return;
        }
        let mut max_total: usize = 0;
        for file in files {
            let long_file = LongFile::new(file);
            let total = long_file.date.format("%b %e %H:%M").to_string().len() - 2;
            if total > max_total {
                max_total = total;
            }
        }

        for file in files {
            let long_file = LongFile::new(file);
            println!(
                "{} {} {} {:>width$} {} {:>4} {} {:>width$} {} {:>width$} {} {:>width$} {} {:>width$} {}",
                self.get_color_for_file_type(&long_file),
                long_file.permissions,
                color::Fg(color::Reset),
                long_file.links,
                color::Fg(color::LightBlue),
                long_file.user,
                color::Fg(color::Reset),
                long_file.group,
                color::Fg(color::LightGreen),
                LongLister::new().format_size(long_file.size),
                color::Fg(color::Reset),
                long_file.date.format("%b %e %H:%M"),
                color::Fg(color::Reset),
                self.get_config_files_colors(&long_file),
                long_file.name,
                width = max_total - 2
            );
        }
    }

    fn get_color_for_file_type(
        &self,
        long_file: &LongFile,
    ) -> termion::color::Fg<termion::color::Rgb> {
        match long_file.file_type.as_str() {
            "file" => color::Fg(color::Rgb(255, 255, 255)),
            "dir" => color::Fg(color::Rgb(0, 255, 255)),
            "link" => color::Fg(color::Rgb(255, 255, 0)),
            _ => color::Fg(color::Rgb(255, 0, 255)),
        }
    }

    fn get_config_files_colors(
        &self,
        long_file: &LongFile,
    ) -> termion::color::Fg<termion::color::Rgb> {
        let config_files_vec = vec![
            ".bashrc",
            ".zshrc",
            ".vimrc",
            ".gitconfig",
            "package.json",
            "Cargo.toml",
            "Gemfile",
            "Rakefile",
            "Makefile",
            "Makefile.lock",
            "docker-compose.yml",
            "docker-compose.yaml",
            "dockerfile",
        ];
        let is_dir = long_file.file_type == "dir";
        for config_file in config_files_vec {
            if long_file.name == config_file {
                return color::Fg(color::Rgb(255, 165, 0));
            }
        }
        if is_dir {
            return color::Fg(color::Rgb(0, 255, 255));
        }
        color::Fg(color::Rgb(255, 255, 255))
    }
}

impl TreeFileFormatter {
    pub fn new() -> TreeFileFormatter {
        TreeFileFormatter
    }

    pub fn build_map(&self, files: &[PathBuf]) -> HashMap<String, Vec<String>> {
        let mut map: HashMap<String, Vec<String>> = HashMap::new();
        for file in files {
            let parent = file.parent().unwrap().to_string_lossy().to_string();
            let child_name = match file.file_name() {
                Some(name) => name.to_string_lossy().to_string(),
                None => continue,
            };
            map.entry(parent).or_insert(Vec::new()).push(child_name);
        }
        let sorted_map: HashMap<String, Vec<String>> = map
            .into_iter()
            .map(|(k, mut v)| {
                v.sort();
                (k, v)
            })
            .collect();

        let merged_map: HashMap<String, Vec<String>> = sorted_map
            .into_iter()
            .map(|(k, v)| {
                let mut new_k = k.clone();
                if new_k == "." {
                    new_k = String::from("root");
                }
                (new_k, v)
            })
            .collect();
        merged_map
    }
}

impl FileFormatter for TreeFileFormatter {
    fn display_files(&self, files: &[PathBuf], _long: Option<bool>) {
        let map = self.build_map(files);
        for (key, value) in map.iter() {
            println!("{}{}:", color::Fg(color::Cyan), key);
            for child in value {
                print!("  ");
                println!("{}{}", color::Fg(color::Reset), child);
            }
        }
    }
}
