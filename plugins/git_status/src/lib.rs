use colored::Colorize;
use lla_plugin_interface::{DecoratedEntry, EntryDecorator, Plugin};
use std::env;
use std::path::Path;
use std::process::Command;

pub struct GitStatusPlugin;

impl Default for GitStatusPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl GitStatusPlugin {
    pub fn new() -> Self {
        GitStatusPlugin
    }

    fn is_git_repo(path: &Path) -> bool {
        let mut current_dir = Some(path);
        while let Some(dir) = current_dir {
            if dir.join(".git").exists() {
                return true;
            }
            current_dir = dir.parent();
        }
        false
    }

    fn get_git_status(path: &Path) -> Option<String> {
        if !Self::is_git_repo(path) {
            return None;
        }

        let path_str = path.to_string_lossy();

        let output = Command::new("git")
            .args(["status", "--porcelain", "--ignored"])
            .arg(&*path_str)
            .current_dir(path.parent().unwrap_or(path))
            .output()
            .ok()?;

        String::from_utf8(output.stdout).ok()
    }

    fn format_git_status(status: &str) -> String {
        let mut status_chars = status.chars().take(2).collect::<Vec<_>>();
        while status_chars.len() < 2 {
            status_chars.push(' ');
        }

        let index_status = status_chars[0];
        let worktree_status = status_chars[1];

        let index_formatted = match index_status {
            'M' => "M".bright_yellow(),
            'A' => "A".bright_green(),
            'D' => "D".bright_red(),
            'R' => "R".bright_purple(),
            'C' => "C".bright_cyan(),
            'U' => "U".bright_magenta(),
            '!' => "!".bright_red(),
            _ => " ".normal(),
        };

        let worktree_formatted = match worktree_status {
            'M' => "M".bright_yellow(),
            '?' => "?".bright_blue(),
            '!' => "!".bright_red(),
            _ => " ".normal(),
        };

        format!("{}{}", index_formatted, worktree_formatted)
    }
}

impl Plugin for GitStatusPlugin {
    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn description(&self) -> &'static str {
        env!("CARGO_PKG_DESCRIPTION")
    }
}

impl EntryDecorator for GitStatusPlugin {
    fn name(&self) -> &'static str {
        env!("CARGO_PKG_NAME")
    }

    fn decorate(&self, entry: &mut DecoratedEntry) {
        if let Some(status) = Self::get_git_status(&entry.path) {
            let formatted_status = Self::format_git_status(&status);
            entry
                .custom_fields
                .insert("git_status".to_string(), formatted_status);
        }
    }

    fn format_field(&self, entry: &DecoratedEntry, _format: &str) -> Option<String> {
        entry.custom_fields.get("git_status").cloned()
    }
}

lla_plugin_interface::declare_plugin!(GitStatusPlugin);
