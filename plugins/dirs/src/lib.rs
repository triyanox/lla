use colored::Colorize;
use lazy_static::lazy_static;
use lla_plugin_interface::{
    proto::{self, plugin_message::Message},
    Plugin,
};
use parking_lot::RwLock;
use prost::Message as _;
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::SystemTime;
use walkdir::WalkDir;

type DirStats = (usize, usize, u64);
type CacheEntry = (SystemTime, DirStats);
type DirCache = HashMap<String, CacheEntry>;

lazy_static! {
    static ref CACHE: RwLock<DirCache> = RwLock::new(HashMap::new());
}

pub struct DirsPlugin;

impl DirsPlugin {
    pub fn new() -> Self {
        DirsPlugin
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

    fn analyze_directory(path: &Path) -> Option<(usize, usize, u64)> {
        let path_str = path.to_string_lossy().to_string();
        if let Ok(metadata) = path.metadata() {
            if let Ok(modified_time) = metadata.modified() {
                let cache = CACHE.read();
                if let Some((cached_time, stats)) = cache.get(&path_str) {
                    if *cached_time >= modified_time {
                        return Some(*stats);
                    }
                }
            }
        }

        let file_count = AtomicUsize::new(0);
        let dir_count = AtomicUsize::new(0);
        let total_size = AtomicU64::new(0);

        WalkDir::new(path)
            .into_iter()
            .par_bridge()
            .filter_map(|e| e.ok())
            .for_each(|entry| {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        file_count.fetch_add(1, Ordering::Relaxed);
                        total_size.fetch_add(metadata.len(), Ordering::Relaxed);
                    } else if metadata.is_dir() {
                        dir_count.fetch_add(1, Ordering::Relaxed);
                    }
                }
            });

        let result = (
            file_count.load(Ordering::Relaxed),
            dir_count.load(Ordering::Relaxed),
            total_size.load(Ordering::Relaxed),
        );

        if let Ok(metadata) = path.metadata() {
            if let Ok(modified_time) = metadata.modified() {
                let mut cache = CACHE.write();
                cache.insert(path_str, (modified_time, result));
            }
        }

        Some(result)
    }

    fn format_size(size: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if size >= GB {
            format!("{:.2} GB", size as f64 / GB as f64)
        } else if size >= MB {
            format!("{:.2} MB", size as f64 / MB as f64)
        } else if size >= KB {
            format!("{:.2} KB", size as f64 / KB as f64)
        } else {
            format!("{} B", size)
        }
    }
}

impl Default for DirsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for DirsPlugin {
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

                if entry.metadata.is_dir {
                    if let Some((file_count, dir_count, total_size)) =
                        Self::analyze_directory(&entry.path)
                    {
                        entry
                            .custom_fields
                            .insert("dir_file_count".to_string(), file_count.to_string());
                        entry
                            .custom_fields
                            .insert("dir_subdir_count".to_string(), dir_count.to_string());
                        entry
                            .custom_fields
                            .insert("dir_total_size".to_string(), Self::format_size(total_size));
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

                if !entry.metadata.is_dir {
                    Message::FieldResponse(proto::FormattedFieldResponse { field: None })
                } else {
                    let formatted = match req.format.as_str() {
                        "long" => {
                            if let (Some(file_count), Some(dir_count), Some(total_size)) = (
                                entry.custom_fields.get("dir_file_count"),
                                entry.custom_fields.get("dir_subdir_count"),
                                entry.custom_fields.get("dir_total_size"),
                            ) {
                                let modified = entry
                                    .path
                                    .metadata()
                                    .ok()
                                    .and_then(|m| m.modified().ok())
                                    .and_then(|t| t.elapsed().ok())
                                    .map(|e| {
                                        let secs = e.as_secs();
                                        if secs < 60 {
                                            format!("{} secs ago", secs)
                                        } else if secs < 3600 {
                                            format!("{} mins ago", secs / 60)
                                        } else if secs < 86400 {
                                            format!("{} hours ago", secs / 3600)
                                        } else {
                                            format!("{} days ago", secs / 86400)
                                        }
                                    })
                                    .unwrap_or_else(|| "unknown time".to_string());

                                Some(format!(
                                    "{} files, {} dirs, {} (modified {})",
                                    file_count.bright_cyan(),
                                    dir_count.bright_green(),
                                    total_size.bright_yellow(),
                                    modified.bright_magenta()
                                ))
                            } else {
                                None
                            }
                        }
                        "default" => {
                            if let (Some(file_count), Some(total_size)) = (
                                entry.custom_fields.get("dir_file_count"),
                                entry.custom_fields.get("dir_total_size"),
                            ) {
                                Some(format!("{} files, {}", file_count, total_size))
                            } else {
                                None
                            }
                        }
                        _ => None,
                    };
                    Message::FieldResponse(proto::FormattedFieldResponse { field: formatted })
                }
            }
            Some(Message::Action(_req)) => Message::ActionResponse(proto::ActionResponse {
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

lla_plugin_interface::declare_plugin!(DirsPlugin);
