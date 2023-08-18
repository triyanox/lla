use crate::ls::LongLister;

use super::file_formater::FileFormatter;
use termion::color;
pub struct TreeFileFormatter;
use std::path::PathBuf;

#[derive(Clone)]
pub struct FSTreeItem {
    pub name: String,
    pub children: Vec<FSTreeItem>,
    pub type_: String,
    pub git_status: Option<String>,
}

impl TreeFileFormatter {
    pub fn new() -> TreeFileFormatter {
        TreeFileFormatter
    }

    pub fn display_files_tree_format(
        &self,
        files: &[PathBuf],
        _long: Option<bool>,
        git: Option<bool>,
    ) {
        let mut root = FSTreeItem {
            name: ".".to_string(),
            children: Vec::new(),
            type_: "dir".to_string(),
            git_status: None,
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
                    let full_path = file.parent().unwrap().join(component_name.clone());
                    let path_buf = PathBuf::from(component_name.clone());

                    let new_node = FSTreeItem {
                        name: component_name.clone(),
                        children: Vec::new(),
                        type_: path_buf
                            .is_dir()
                            .then(|| "dir".to_string())
                            .unwrap_or("file".to_string()),
                        git_status: Some(self.get_git_status(full_path.to_str().unwrap())),
                    };
                    current_node.children.push(new_node);
                    let new_index = current_node.children.len() - 1;
                    current_node = &mut current_node.children[new_index];
                }
            }
        }
        self.print_tree(&root, 0, true, git);
    }

    fn get_file_color(&self, file_type: &str) -> termion::color::Fg<termion::color::Rgb> {
        match file_type {
            "file" => color::Fg(color::Rgb(255, 255, 255)),
            "dir" => color::Fg(color::Rgb(0, 255, 255)),
            "link" => color::Fg(color::Rgb(255, 255, 0)),
            _ => color::Fg(color::Rgb(255, 0, 255)),
        }
    }

    fn print_tree(&self, tree: &FSTreeItem, depth: usize, is_last: bool, git: Option<bool>) {
        let mut prefix = String::new();

        for _ in 0..depth {
            prefix.push_str("│   ");
        }
        if depth > 0 {
            prefix.push_str(if is_last { "└── " } else { "├── " });
        }

        prefix.push_str(&tree.name);
        println!("{}", prefix);

        for (i, child) in tree.children.iter().enumerate() {
            let type_color = self.get_file_color(&child.type_);
            let is_last_child = i == tree.children.len() - 1;
            let mut child = child.clone();
            let git_status = child.git_status.clone().unwrap();

            let status_color_map = vec![
                ("?", color::Fg(color::Rgb(255, 255, 0))),
                ("A", color::Fg(color::Rgb(0, 255, 0))),
                ("M", color::Fg(color::Rgb(255, 0, 0))),
                ("R", color::Fg(color::Rgb(255, 0, 255))),
                ("C", color::Fg(color::Rgb(255, 0, 255))),
                ("D", color::Fg(color::Rgb(255, 0, 0))),
                ("!", color::Fg(color::Rgb(255, 0, 255))),
            ];

            let mut status_color = color::Fg(color::Rgb(255, 255, 255));
            let reset_color = color::Fg(color::Rgb(255, 255, 255));
            for (status, color) in status_color_map {
                if git_status.contains(status) {
                    status_color = color;
                    break;
                }
            }
            child.name = if is_last_child {
                if let Some(git) = git {
                    if git {
                        format!(
                            "{}{}{}{}{}{}{}{}",
                            "└── ",
                            type_color,
                            child.name,
                            reset_color,
                            " ",
                            status_color,
                            git_status,
                            reset_color,
                        )
                    } else {
                        format!("{}{}{}{}", "└── ", type_color, child.name, reset_color)
                    }
                } else {
                    format!("{}{}{}{}", "└── ", type_color, child.name, reset_color)
                }
            } else {
                if let Some(git) = git {
                    if git {
                        format!(
                            "{}{}{}{}{}{}{}{}",
                            "├── ",
                            type_color,
                            child.name,
                            reset_color,
                            " ",
                            status_color,
                            git_status,
                            reset_color,
                        )
                    } else {
                        format!("{}{}{}{}", "├── ", type_color, child.name, reset_color)
                    }
                } else {
                    format!("{}{}{}{}", "├── ", type_color, child.name, reset_color)
                }
            };
            self.print_tree(&child, depth + 1, is_last_child, git);
        }
    }

    fn get_git_status(&self, path: &str) -> String {
        let path = path.replace("./", "");
        let git_status = LongLister::new().get_git_status(&path);
        return git_status;
    }
}

impl FileFormatter for TreeFileFormatter {
    fn display_files(&self, files: &[PathBuf], _long: Option<bool>, git: Option<bool>) {
        self.display_files_tree_format(files, _long, git);
    }
}
