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
        let mut current_dir = path;
        loop {
            if current_dir.join(".git").exists() {
                return true;
            }
            match current_dir.parent() {
                Some(parent) => current_dir = parent,
                None => return false,
            }
        }
    }

    fn get_git_status(path: &Path) -> Option<String> {
        if !Self::is_git_repo(path) {
            return None;
        }

        let output = Command::new("git")
            .args(["status", "--porcelain", "--ignored", path.to_str().unwrap()])
            .output()
            .ok()?;

        let status = String::from_utf8(output.stdout).ok()?;

        Some(status)
    }

    fn format_git_status(status: &str) -> String {
        let mut chars = status.chars();
        let index_status = chars.next().unwrap_or(' ');
        let worktree_status = chars.next().unwrap_or(' ');

        let index_formatted = match index_status {
            'M' => "M".bright_yellow().to_string(),
            'A' => "A".bright_green().to_string(),
            'D' => "D".bright_red().to_string(),
            'R' => "R".bright_purple().to_string(),
            'C' => "C".bright_cyan().to_string(),
            'U' => "U".bright_magenta().to_string(),
            '!' => "!".bright_red().to_string(),
            _ => " ".normal().to_string(),
        };

        let worktree_formatted = match worktree_status {
            'M' => "M".bright_yellow().to_string(),
            '?' => "?".bright_blue().to_string(),
            '!' => "!".bright_red().to_string(),
            _ => " ".normal().to_string(),
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
