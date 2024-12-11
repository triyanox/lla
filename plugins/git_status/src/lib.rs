use colored::Colorize;
use lla_plugin_interface::{
    proto::{self, plugin_message::Message},
    Plugin,
};
use prost::Message as _;
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

    fn get_git_info(path: &Path) -> Option<(String, String, String)> {
        if !Self::is_git_repo(path) {
            return None;
        }

        let path_str = path.to_string_lossy();
        let parent = path.parent().unwrap_or(path);

        let status_output = Command::new("git")
            .args(["status", "--porcelain", "--ignored"])
            .arg(&*path_str)
            .current_dir(parent)
            .output()
            .ok()?;

        let branch_output = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(parent)
            .output()
            .ok()?;

        let commit_output = Command::new("git")
            .args(["log", "-1", "--format=%h %s"])
            .current_dir(parent)
            .output()
            .ok()?;

        let status = String::from_utf8(status_output.stdout).ok()?;
        let branch = String::from_utf8(branch_output.stdout)
            .ok()?
            .trim()
            .to_string();
        let commit = String::from_utf8(commit_output.stdout)
            .ok()?
            .trim()
            .to_string();

        Some((status, branch, commit))
    }

    fn format_git_status(status: &str) -> (String, usize, usize, usize, usize) {
        let mut staged = 0;
        let mut modified = 0;
        let mut untracked = 0;
        let mut conflicts = 0;

        let mut formatted_entries = Vec::new();

        for line in status.lines() {
            let status_chars: Vec<char> = line.chars().take(2).collect();
            let index_status = status_chars.get(0).copied().unwrap_or(' ');
            let worktree_status = status_chars.get(1).copied().unwrap_or(' ');

            match (index_status, worktree_status) {
                ('M', ' ') => {
                    staged += 1;
                    formatted_entries.push(format!("{} staged", "✓".bright_green()));
                }
                (' ', 'M') => {
                    modified += 1;
                    formatted_entries.push(format!("{} modified", "±".bright_yellow()));
                }
                ('M', 'M') => {
                    staged += 1;
                    modified += 1;
                    formatted_entries.push(format!("{} staged & modified", "±".bright_yellow()));
                }
                ('A', ' ') => {
                    staged += 1;
                    formatted_entries.push(format!("{} new file", "✚".bright_green()));
                }
                ('D', ' ') | (' ', 'D') => {
                    modified += 1;
                    formatted_entries.push(format!("{} deleted", "✖".bright_red()));
                }
                ('R', _) => {
                    staged += 1;
                    formatted_entries.push(format!("{} renamed", "➜".bright_purple()));
                }
                ('C', _) => {
                    staged += 1;
                    formatted_entries.push(format!("{} copied", "↠".bright_cyan()));
                }
                ('U', _) | (_, 'U') => {
                    conflicts += 1;
                    formatted_entries.push(format!("{} conflict", "⚡".bright_magenta()));
                }
                (' ', '?') => {
                    untracked += 1;
                    formatted_entries.push(format!("{} untracked", "?".bright_blue()));
                }
                _ => {}
            }
        }

        let status_summary = if formatted_entries.is_empty() {
            "clean".bright_green().to_string()
        } else {
            formatted_entries.join(", ")
        };

        (status_summary, staged, modified, untracked, conflicts)
    }

    fn encode_error(&self, error: &str) -> Vec<u8> {
        use prost::Message;
        let error_msg = lla_plugin_interface::proto::PluginMessage {
            message: Some(
                lla_plugin_interface::proto::plugin_message::Message::ErrorResponse(
                    error.to_string(),
                ),
            ),
        };
        let mut buf = bytes::BytesMut::with_capacity(error_msg.encoded_len());
        error_msg.encode(&mut buf).unwrap();
        buf.to_vec()
    }
}

impl Plugin for GitStatusPlugin {
    fn handle_raw_request(&mut self, request: &[u8]) -> Vec<u8> {
        let proto_msg = match proto::PluginMessage::decode(request) {
            Ok(msg) => msg,
            Err(e) => {
                let error_msg = proto::PluginMessage {
                    message: Some(Message::ErrorResponse(format!(
                        "Failed to decode request: {}",
                        e
                    ))),
                };
                let mut buf = bytes::BytesMut::with_capacity(error_msg.encoded_len());
                error_msg.encode(&mut buf).unwrap();
                return buf.to_vec();
            }
        };

        let response_msg = match proto_msg.message {
            Some(Message::GetName(_)) => Message::NameResponse(env!("CARGO_PKG_NAME").to_string()),
            Some(Message::GetVersion(_)) => {
                Message::VersionResponse(env!("CARGO_PKG_VERSION").to_string())
            }
            Some(Message::GetDescription(_)) => {
                Message::DescriptionResponse(env!("CARGO_PKG_DESCRIPTION").to_string())
            }
            Some(Message::GetSupportedFormats(_)) => {
                Message::FormatsResponse(proto::SupportedFormatsResponse {
                    formats: vec!["default".to_string(), "long".to_string()],
                })
            }
            Some(Message::Decorate(entry)) => {
                let mut entry = match lla_plugin_interface::DecoratedEntry::try_from(entry.clone())
                {
                    Ok(e) => e,
                    Err(e) => {
                        return self.encode_error(&format!("Failed to convert entry: {}", e));
                    }
                };

                if let Some((status, branch, commit)) = Self::get_git_info(&entry.path) {
                    let (status_summary, staged, modified, untracked, conflicts) =
                        Self::format_git_status(&status);
                    entry
                        .custom_fields
                        .insert("git_status".to_string(), status_summary);
                    entry.custom_fields.insert("git_branch".to_string(), branch);
                    entry.custom_fields.insert("git_commit".to_string(), commit);
                    entry
                        .custom_fields
                        .insert("git_staged".to_string(), staged.to_string());
                    entry
                        .custom_fields
                        .insert("git_modified".to_string(), modified.to_string());
                    entry
                        .custom_fields
                        .insert("git_untracked".to_string(), untracked.to_string());
                    entry
                        .custom_fields
                        .insert("git_conflicts".to_string(), conflicts.to_string());
                }
                Message::DecoratedResponse(entry.into())
            }
            Some(Message::FormatField(req)) => {
                let entry = match req.entry {
                    Some(e) => match lla_plugin_interface::DecoratedEntry::try_from(e) {
                        Ok(entry) => entry,
                        Err(e) => {
                            return self.encode_error(&format!("Failed to convert entry: {}", e));
                        }
                    },
                    None => return self.encode_error("Missing entry in format field request"),
                };

                let formatted = match req.format.as_str() {
                    "long" => {
                        if let (Some(_status), Some(branch), Some(commit)) = (
                            entry.custom_fields.get("git_status"),
                            entry.custom_fields.get("git_branch"),
                            entry.custom_fields.get("git_commit"),
                        ) {
                            let stats = [
                                entry.custom_fields.get("git_staged").and_then(|s| {
                                    let count: usize = s.parse().unwrap_or(0);
                                    if count > 0 {
                                        Some(format!("{} staged", count.to_string().bright_green()))
                                    } else {
                                        None
                                    }
                                }),
                                entry.custom_fields.get("git_modified").and_then(|s| {
                                    let count: usize = s.parse().unwrap_or(0);
                                    if count > 0 {
                                        Some(format!(
                                            "{} modified",
                                            count.to_string().bright_yellow()
                                        ))
                                    } else {
                                        None
                                    }
                                }),
                                entry.custom_fields.get("git_untracked").and_then(|s| {
                                    let count: usize = s.parse().unwrap_or(0);
                                    if count > 0 {
                                        Some(format!(
                                            "{} untracked",
                                            count.to_string().bright_blue()
                                        ))
                                    } else {
                                        None
                                    }
                                }),
                                entry.custom_fields.get("git_conflicts").and_then(|s| {
                                    let count: usize = s.parse().unwrap_or(0);
                                    if count > 0 {
                                        Some(format!(
                                            "{} conflicts",
                                            count.to_string().bright_red()
                                        ))
                                    } else {
                                        None
                                    }
                                }),
                            ]
                            .into_iter()
                            .filter_map(|s| s)
                            .collect::<Vec<_>>();

                            let status_line = if stats.is_empty() {
                                "Status:   working tree clean".bright_green().to_string()
                            } else {
                                format!("Status:   {}", stats.join(", "))
                            };

                            let branch_symbol = "⎇".bright_blue();
                            let commit_parts: Vec<&str> = commit.split_whitespace().collect();
                            let (commit_hash, commit_msg) = match commit_parts.split_first() {
                                Some((hash, msg)) => (*hash, msg.join(" ")),
                                None => (commit.as_str(), String::new()),
                            };

                            Some(format!(
                                "\n{}\n{}\n{}\n{}",
                                format!("Branch:   {} {}", branch_symbol, branch.bright_cyan()),
                                format!(
                                    "Commit:   {} {}",
                                    commit_hash.bright_yellow(),
                                    commit_msg.bright_white()
                                ),
                                status_line,
                                format!(
                                    "Repo:     {}",
                                    if !stats.is_empty() {
                                        "has changes".bright_yellow()
                                    } else {
                                        "clean".bright_green()
                                    }
                                )
                            ))
                        } else {
                            None
                        }
                    }
                    "default" => entry.custom_fields.get("git_status").cloned(),
                    _ => None,
                };
                Message::FieldResponse(proto::FormattedFieldResponse { field: formatted })
            }
            Some(Message::Action(_)) => Message::ActionResponse(proto::ActionResponse {
                success: true,
                error: None,
            }),
            _ => Message::ErrorResponse("Invalid request type".to_string()),
        };

        let response = proto::PluginMessage {
            message: Some(response_msg),
        };
        let mut buf = bytes::BytesMut::with_capacity(response.encoded_len());
        response.encode(&mut buf).unwrap();
        buf.to_vec()
    }
}

lla_plugin_interface::declare_plugin!(GitStatusPlugin);
