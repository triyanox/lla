use colored::Colorize;
use lla_plugin_interface::{
    proto::{self, plugin_message::Message},
    Plugin,
};
use prost::Message as _;
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

impl Plugin for FileHashPlugin {
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

                if entry.path.is_file() {
                    if let Some((sha1, sha256)) = Self::calculate_hashes(&entry.path) {
                        entry.custom_fields.insert("sha1".to_string(), sha1);
                        entry.custom_fields.insert("sha256".to_string(), sha256);
                    }
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

impl Default for FileHashPlugin {
    fn default() -> Self {
        Self::new()
    }
}

lla_plugin_interface::declare_plugin!(FileHashPlugin);
