use colored::Colorize;
use lla_plugin_interface::{Plugin, PluginRequest, PluginResponse};
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
}

impl Plugin for GitStatusPlugin {
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
                PluginResponse::Decorated(entry)
            }
            PluginRequest::FormatField(entry, format) => {
                let formatted = match format.as_str() {
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
                PluginResponse::FormattedField(formatted)
            }
            PluginRequest::PerformAction(_, _) => PluginResponse::ActionResult(Ok(())),
        }
    }
}

lla_plugin_interface::declare_plugin!(GitStatusPlugin);
