use colored::*;
use lla_plugin_interface::{
    proto::{self, plugin_message::Message},
    Plugin,
};
use prost::Message as _;
use std::time::SystemTime;

pub struct FileMetadataPlugin;

impl FileMetadataPlugin {
    pub fn new() -> Self {
        FileMetadataPlugin
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

    fn format_timestamp(timestamp: SystemTime) -> String {
        let datetime: chrono::DateTime<chrono::Local> = timestamp.into();
        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    }
}

impl Plugin for FileMetadataPlugin {
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

                entry.custom_fields.insert(
                    "accessed".to_string(),
                    Self::format_timestamp(
                        SystemTime::UNIX_EPOCH
                            + std::time::Duration::from_secs(entry.metadata.accessed),
                    ),
                );
                entry.custom_fields.insert(
                    "modified".to_string(),
                    Self::format_timestamp(
                        SystemTime::UNIX_EPOCH
                            + std::time::Duration::from_secs(entry.metadata.modified),
                    ),
                );
                entry.custom_fields.insert(
                    "created".to_string(),
                    Self::format_timestamp(
                        SystemTime::UNIX_EPOCH
                            + std::time::Duration::from_secs(entry.metadata.created),
                    ),
                );
                entry
                    .custom_fields
                    .insert("uid".to_string(), entry.metadata.uid.to_string());
                entry
                    .custom_fields
                    .insert("gid".to_string(), entry.metadata.gid.to_string());
                entry
                    .custom_fields
                    .insert("size".to_string(), entry.metadata.size.to_string());
                entry.custom_fields.insert(
                    "permissions".to_string(),
                    format!("{:o}", entry.metadata.permissions),
                );

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
                        match (
                            entry.custom_fields.get("accessed"),
                            entry.custom_fields.get("modified"),
                            entry.custom_fields.get("created"),
                            entry.custom_fields.get("uid"),
                            entry.custom_fields.get("gid"),
                            entry.custom_fields.get("size"),
                            entry.custom_fields.get("permissions"),
                        ) {
                            (
                                Some(accessed),
                                Some(modified),
                                Some(created),
                                Some(uid),
                                Some(gid),
                                Some(size),
                                Some(permissions),
                            ) => Some(format!(
                                "\n{}\n{}\n{}\n{}\n{}\n{}",
                                format!("Accessed: {}", accessed.blue()),
                                format!("Modified: {}", modified.green()),
                                format!("Created:  {}", created.yellow()),
                                format!("UID/GID:  {}/{}", uid.magenta(), gid.magenta()),
                                format!("Size:     {}", size.cyan()),
                                format!("Perms:    {}", permissions.red())
                            )),
                            _ => None,
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

impl Default for FileMetadataPlugin {
    fn default() -> Self {
        Self::new()
    }
}

lla_plugin_interface::declare_plugin!(FileMetadataPlugin);
