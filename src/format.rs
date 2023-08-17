use crate::error::LlaError;
use crate::ls::LongLister;
use std::process::exit;
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
            "pnpm-lock.yaml",
            "pnpmfile.js",
            ".npmrc",
            "yarn.lock",
            "yarn.lock",
        ];
        let is_dir = long_file.file_type == "dir";
        for config_file in config_files_vec {
            if long_file.name == config_file {
                return color::Fg(color::Rgb(255, 0, 255));
            }
        }
        if is_dir {
            return color::Fg(color::Rgb(0, 255, 255));
        }
        color::Fg(color::Rgb(255, 255, 255))
    }
}

#[derive(Clone)]
pub struct FSTreeItem {
    pub name: String,
    pub children: Vec<FSTreeItem>,
    pub type_: String,
}

impl TreeFileFormatter {
    pub fn new() -> TreeFileFormatter {
        TreeFileFormatter
    }

    pub fn display_files_tree_format(&self, files: &[PathBuf]) {
        let mut root = FSTreeItem {
            name: ".".to_string(),
            children: Vec::new(),
            type_: "dir".to_string(),
        };

        for file in files {
            let path = file.clone();
            let mut current_node = &mut root;

            for component in path.components() {
                let component_name = component.as_os_str().to_str().unwrap().to_string();
                let child_index = current_node
                    .children
                    .iter()
                    .position(|c| c.name == component_name);

                if let Some(index) = child_index {
                    current_node = &mut current_node.children[index];
                } else {
                    let path_buf = PathBuf::from(component_name.clone());
                    let new_node = FSTreeItem {
                        name: component_name.clone(),
                        children: Vec::new(),
                        type_: path_buf
                            .is_dir()
                            .then(|| "dir".to_string())
                            .unwrap_or("file".to_string()),
                    };
                    current_node.children.push(new_node);
                    let new_index = current_node.children.len() - 1;
                    current_node = &mut current_node.children[new_index];
                }
            }
        }
        self.print_tree(&root, 0, true);
    }

    fn get_file_color(&self, file_type: &str) -> termion::color::Fg<termion::color::Rgb> {
        match file_type {
            "file" => color::Fg(color::Rgb(255, 255, 255)),
            "dir" => color::Fg(color::Rgb(0, 255, 255)),
            "link" => color::Fg(color::Rgb(255, 255, 0)),
            _ => color::Fg(color::Rgb(255, 0, 255)),
        }
    }

    fn print_tree(&self, tree: &FSTreeItem, depth: usize, is_last: bool) {
        let mut prefix = String::new();

        for _ in 0..depth {
            prefix.push_str("│   ");
        }
        if depth > 0 {
            prefix.push_str(if is_last { "└── " } else { "├── " });
        }

        prefix.push_str(&tree.name);
        let reset_color = color::Fg(color::Reset);
        println!("{}", prefix);

        for (i, child) in tree.children.iter().enumerate() {
            let type_color = self.get_file_color(&child.type_);
            let is_last_child = i == tree.children.len() - 1;
            let mut child = child.clone();
            child.name = if is_last_child {
                format!("{}{}{}{}", "└── ", type_color, child.name, reset_color)
            } else {
                format!("{}{}{}{}", "├── ", type_color, child.name, reset_color)
            };
            self.print_tree(&child, depth + 1, is_last_child);
        }
    }
}

impl FileFormatter for TreeFileFormatter {
    fn display_files(&self, files: &[PathBuf], _long: Option<bool>) {
        self.display_files_tree_format(files);
    }
}
