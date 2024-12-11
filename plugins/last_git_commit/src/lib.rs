use colored::Colorize;
use lla_plugin_interface::{
    proto::{self, plugin_message::Message},
    Plugin,
};
use prost::Message as _;
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

impl Plugin for LastGitCommitPlugin {
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

                if let Some((commit_hash, author, time)) = Self::get_last_commit_info(&entry.path) {
                    entry
                        .custom_fields
                        .insert("commit_hash".to_string(), commit_hash);
                    entry
                        .custom_fields
                        .insert("commit_author".to_string(), author);
                    entry.custom_fields.insert("commit_time".to_string(), time);
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

impl Default for LastGitCommitPlugin {
    fn default() -> Self {
        Self::new()
    }
}

lla_plugin_interface::declare_plugin!(LastGitCommitPlugin);
