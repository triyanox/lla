use colored::Colorize;
use lla_plugin_interface::{Plugin, PluginRequest, PluginResponse};
use sha1::Sha1;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Read};

pub struct FileHashPlugin;

impl FileHashPlugin {
    pub fn new() -> Self {
        FileHashPlugin
    }

    fn calculate_hashes(path: &std::path::Path) -> Option<(String, String)> {
        let file = File::open(path).ok()?;
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).ok()?;

        let sha1 = format!("{:x}", Sha1::digest(&buffer));
        let sha256 = format!("{:x}", Sha256::digest(&buffer));

        Some((sha1, sha256))
    }
}

impl Plugin for FileHashPlugin {
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
                if entry.path.is_file() {
                    if let Some((sha1, sha256)) = Self::calculate_hashes(&entry.path) {
                        entry.custom_fields.insert("sha1".to_string(), sha1);
                        entry.custom_fields.insert("sha256".to_string(), sha256);
                    }
                }
                PluginResponse::Decorated(entry)
            }
            PluginRequest::FormatField(entry, format) => {
                let formatted = match format.as_str() {
                    "long" | "default" => {
                        if entry.path.is_dir() {
                            None
                        } else {
                            let sha1 = entry
                                .custom_fields
                                .get("sha1")
                                .map(|s| s[..8].to_string())
                                .unwrap_or_default();
                            let sha256 = entry
                                .custom_fields
                                .get("sha256")
                                .map(|s| s[..8].to_string())
                                .unwrap_or_default();
                            Some(format!(
                                "\n{} {} {}{}\n{} {} {}{}",
                                "┌".bright_black(),
                                "SHA1".bright_green().bold(),
                                "→".bright_black(),
                                sha1.green(),
                                "└".bright_black(),
                                "SHA256".bright_yellow().bold(),
                                "→".bright_black(),
                                sha256.yellow()
                            ))
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

impl Default for FileHashPlugin {
    fn default() -> Self {
        Self::new()
    }
}

lla_plugin_interface::declare_plugin!(FileHashPlugin);
