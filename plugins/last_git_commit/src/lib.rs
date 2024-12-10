use colored::Colorize;
use lla_plugin_interface::{Plugin, PluginRequest, PluginResponse};
use std::path::Path;
use std::process::Command;

pub struct LastGitCommitPlugin;

impl LastGitCommitPlugin {
    pub fn new() -> Self {
        LastGitCommitPlugin
    }

    fn get_last_commit_info(path: &Path) -> Option<(String, String, String)> {
        let output = Command::new("git")
            .args(["log", "-1", "--format=%h|%an|%ar", "--", path.to_str()?])
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
    fn handle_request(&mut self, request: PluginRequest) -> PluginResponse {
        match request {
            PluginRequest::GetName => PluginResponse::Name(env!("CARGO_PKG_NAME").to_string()),
            PluginRequest::GetVersion => {
                PluginResponse::Version(env!("CARGO_PKG_VERSION").to_string())
            }
            PluginRequest::GetDescription => {
                PluginResponse::Description(env!("CARGO_PKG_DESCRIPTION").to_string())
            }
            PluginRequest::GetSupportedFormats => {
                PluginResponse::SupportedFormats(vec!["default".to_string(), "long".to_string()])
            }
            PluginRequest::Decorate(mut entry) => {
                if let Some((commit_hash, author, time)) = Self::get_last_commit_info(&entry.path) {
                    entry
                        .custom_fields
                        .insert("commit_hash".to_string(), commit_hash);
                    entry
                        .custom_fields
                        .insert("commit_author".to_string(), author);
                    entry.custom_fields.insert("commit_time".to_string(), time);
                }
                PluginResponse::Decorated(entry)
            }
            PluginRequest::FormatField(entry, format) => {
                let formatted = match format.as_str() {
                    "long" => {
                        if let (Some(hash), Some(author), Some(time)) = (
                            entry.custom_fields.get("commit_hash"),
                            entry.custom_fields.get("commit_author"),
                            entry.custom_fields.get("commit_time"),
                        ) {
                            Some(format!(
                                "Last commit: {} by {} {}",
                                hash.bright_yellow(),
                                author.bright_cyan(),
                                time.bright_green()
                            ))
                        } else {
                            None
                        }
                    }
                    "default" => {
                        if let (Some(hash), Some(time)) = (
                            entry.custom_fields.get("commit_hash"),
                            entry.custom_fields.get("commit_time"),
                        ) {
                            Some(format!("Commit: {} {}", hash, time))
                        } else {
                            None
                        }
                    }
                    _ => None,
                };
                PluginResponse::FormattedField(formatted)
            }
            PluginRequest::PerformAction(_, _) => PluginResponse::ActionResult(Ok(())),
        }
    }
}

impl Default for LastGitCommitPlugin {
    fn default() -> Self {
        Self::new()
    }
}

lla_plugin_interface::declare_plugin!(LastGitCommitPlugin);
