use super::file_formater::FileFormatter;
use super::LongFile;
use crate::ls::LongLister;
use std::path::PathBuf;
use termion::{color, style};

pub struct DefaultFileFormatter;

impl DefaultFileFormatter {
    pub fn new() -> DefaultFileFormatter {
        DefaultFileFormatter
    }
}

impl FileFormatter for DefaultFileFormatter {
    fn display_files(&self, files: &[PathBuf], long: Option<bool>, git: Option<bool>) {
        if let Some(true) = long {
            if let Some(true) = git {
                self.display_files_git_format(files);
            } else {
                self.display_files_long_format(files);
            }
        } else {
            self.display_files_default_format(files);
        }
    }
}

impl DefaultFileFormatter {
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

    fn display_files_git_format(&self, files: &[PathBuf]) {
        if files.is_empty() {
            return;
        }

        let max_total = self.calculate_max_total(files);

        for file in files {
            let long_file = LongFile::new(file);
            let git_status = self.get_git_status(&long_file.name);
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
            for (status, color) in status_color_map {
                if git_status.contains(status) {
                    status_color = color;
                    break;
                }
            }

            println!(
                "{} {} {} {:>width$} {} {:>4} {} {:>width$} {} {:>width$} {} {:>width$} {} {:>width$} {} {} {} {}",
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
                status_color,
                git_status,
                color::Fg(color::Reset),
                self.get_config_files_colors(&long_file),
                long_file.name,
                width = max_total - 2
            );
        }
    }
    fn calculate_max_total(&self, files: &[PathBuf]) -> usize {
        let mut max_total: usize = 0;
        for file in files {
            let long_file = LongFile::new(file);
            let total = long_file.date.format("%b %e %H:%M").to_string().len() - 2;
            if total > max_total {
                max_total = total;
            }
        }
        max_total
    }

    fn get_git_status(&self, file_name: &str) -> String {
        let git_status = LongLister::new().get_git_status(file_name);
        return git_status;
    }
}
