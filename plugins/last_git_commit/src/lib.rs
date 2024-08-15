use colored::Colorize;
use lla_plugin_interface::{DecoratedEntry, EntryDecorator, Plugin};
use std::path::Path;
use std::process::Command;

pub struct LastGitCommitPlugin;

impl LastGitCommitPlugin {
    pub fn new() -> Self {
        LastGitCommitPlugin
    }

    fn get_last_commit_info(path: &Path) -> Option<(String, String, String)> {
        let output = Command::new("git")
            .args(&["log", "-1", "--format=%h|%an|%ar", "--", path.to_str()?])
            .output()
            .ok()?;

        let output_str = String::from_utf8(output.stdout).ok()?;
        let parts: Vec<&str> = output_str.trim().split('|').collect();

        if parts.len() == 3 {
            Some((
                parts[0].to_string(),
                parts[1].to_string(),
                parts[2].to_string(),
            ))
        } else {
            None
        }
    }
}

impl Plugin for LastGitCommitPlugin {
    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn description(&self) -> &'static str {
        env!("CARGO_PKG_DESCRIPTION")
    }
}

impl EntryDecorator for LastGitCommitPlugin {
    fn name(&self) -> &'static str {
        env!("CARGO_PKG_NAME")
    }

    fn decorate(&self, entry: &mut DecoratedEntry) {
        if let Some((commit_hash, author, time)) = Self::get_last_commit_info(&entry.path) {
            entry
                .custom_fields
                .insert("commit_hash".to_string(), commit_hash);
            entry
                .custom_fields
                .insert("commit_author".to_string(), author);
            entry.custom_fields.insert("commit_time".to_string(), time);
        }
    }

    fn format_field(&self, entry: &DecoratedEntry, format: &str) -> Option<String> {
        match format {
            "long" => {
                let hash = entry.custom_fields.get("commit_hash")?;
                let author = entry.custom_fields.get("commit_author")?;
                let time = entry.custom_fields.get("commit_time")?;
                Some(format!(
                    "Last commit: {} by {} {}",
                    hash.bright_yellow(),
                    author.bright_cyan(),
                    time.bright_green()
                ))
            }
            "default" | "tree" => {
                let hash = entry.custom_fields.get("commit_hash")?;
                let time = entry.custom_fields.get("commit_time")?;
                Some(format!("Commit: {} {}", hash, time))
            }
            _ => None,
        }
    }

    fn supported_formats(&self) -> Vec<&'static str> {
        vec!["default", "long", "tree"]
    }
}

lla_plugin_interface::declare_plugin!(LastGitCommitPlugin);
