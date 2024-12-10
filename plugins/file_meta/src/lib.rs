use colored::*;
use lla_plugin_interface::{Plugin, PluginRequest, PluginResponse};
use std::time::SystemTime;

pub struct FileMetadataPlugin;

impl FileMetadataPlugin {
    pub fn new() -> Self {
        FileMetadataPlugin
    }

    fn format_timestamp(timestamp: SystemTime) -> String {
        let datetime: chrono::DateTime<chrono::Local> = timestamp.into();
        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    }
}

impl Plugin for FileMetadataPlugin {
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

                PluginResponse::Decorated(entry)
            }
            PluginRequest::FormatField(entry, format) => {
                let formatted = match format.as_str() {
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
                PluginResponse::FormattedField(formatted)
            }
            PluginRequest::PerformAction(_, _) => PluginResponse::ActionResult(Ok(())),
        }
    }
}

impl Default for FileMetadataPlugin {
    fn default() -> Self {
        Self::new()
    }
}

lla_plugin_interface::declare_plugin!(FileMetadataPlugin);
