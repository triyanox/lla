use colored::Colorize;
use lazy_static::lazy_static;
use lla_plugin_interface::{
    proto::{self, plugin_message::Message},
    DecoratedEntry, Plugin,
};
use parking_lot::RwLock;
use prost::Message as _;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Clone)]
struct FileInfo {
    path: PathBuf,
    modified: SystemTime,
}

lazy_static! {
    static ref CACHE: RwLock<HashMap<String, Vec<FileInfo>>> = RwLock::new(HashMap::new());
}

pub struct DuplicateFileDetectorPlugin;

impl Default for DuplicateFileDetectorPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl DuplicateFileDetectorPlugin {
    pub fn new() -> Self {
        DuplicateFileDetectorPlugin
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

    fn get_file_hash(path: &Path) -> Option<String> {
        let mut file = File::open(path).ok()?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            match file.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => hasher.update(&buffer[..n]),
                Err(_) => return None,
            }
        }
        Some(format!("{:x}", hasher.finalize()))
    }

    fn process_entry(&self, mut entry: DecoratedEntry) -> DecoratedEntry {
        if !entry.metadata.is_file {
            return entry;
        }

        if let Some(hash) = Self::get_file_hash(&entry.path) {
            let mut cache = CACHE.write();
            let entries = cache.entry(hash).or_default();

            let modified = entry
                .path
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .unwrap_or_else(|| SystemTime::now());

            if !entries.iter().any(|f| f.path == entry.path) {
                entries.push(FileInfo {
                    path: entry.path.clone(),
                    modified,
                });
            }

            if entries.len() > 1 {
                let oldest = entries.iter().min_by_key(|f| f.modified).unwrap();

                if oldest.path == entry.path {
                    entry
                        .custom_fields
                        .insert("has_duplicates".to_string(), "true".to_string());

                    let duplicate_paths: Vec<String> = entries
                        .iter()
                        .filter(|f| f.path != oldest.path)
                        .map(|f| f.path.to_string_lossy().to_string())
                        .collect();

                    entry
                        .custom_fields
                        .insert("duplicate_paths".to_string(), duplicate_paths.join(", "));
                } else {
                    entry
                        .custom_fields
                        .insert("is_duplicate".to_string(), "true".to_string());

                    entry.custom_fields.insert(
                        "original_path".to_string(),
                        oldest.path.to_string_lossy().to_string(),
                    );
                }
            }
        }

        entry
    }
}

impl Plugin for DuplicateFileDetectorPlugin {
    fn handle_raw_request(&mut self, request: &[u8]) -> Vec<u8> {
        let proto_msg = match proto::PluginMessage::decode(request) {
            Ok(msg) => msg,
            Err(e) => {
                return self.encode_error(&format!("Failed to decode request: {}", e));
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
                let entry = match DecoratedEntry::try_from(entry.clone()) {
                    Ok(e) => e,
                    Err(e) => {
                        return self.encode_error(&format!("Failed to convert entry: {}", e));
                    }
                };
                Message::DecoratedResponse(self.process_entry(entry).into())
            }
            Some(Message::FormatField(req)) => {
                let entry = match req.entry {
                    Some(e) => match DecoratedEntry::try_from(e) {
                        Ok(entry) => entry,
                        Err(e) => {
                            return self.encode_error(&format!("Failed to convert entry: {}", e));
                        }
                    },
                    None => return self.encode_error("Missing entry in format field request"),
                };

                let formatted = if let Some(_) = entry.custom_fields.get("has_duplicates") {
                    match req.format.as_str() {
                        "long" => Some(format!(
                            "{} {}",
                            "HAS DUPLICATES".bright_yellow(),
                            format!(
                                "copies: {}",
                                entry.custom_fields.get("duplicate_paths").unwrap()
                            )
                            .bright_cyan()
                        )),
                        "default" => Some(format!("{}", "HAS DUPLICATES".bright_yellow())),
                        _ => None,
                    }
                } else {
                    let cache = CACHE.read();
                    let mut original_path = None;
                    let is_duplicate = cache.values().any(|entries| {
                        if let Some(oldest) = entries.iter().min_by_key(|f| f.modified) {
                            let is_dup = entries
                                .iter()
                                .any(|f| f.path == entry.path && oldest.path != entry.path);
                            if is_dup {
                                original_path = Some(oldest.path.to_string_lossy().to_string());
                            }
                            is_dup
                        } else {
                            false
                        }
                    });

                    if is_duplicate {
                        match req.format.as_str() {
                            "long" => Some(format!(
                                "{} {}",
                                "DUPLICATE".bright_red(),
                                format!("of: {}", original_path.unwrap()).bright_cyan()
                            )),
                            "default" => Some(format!("{}", "DUPLICATE".bright_red())),
                            _ => None,
                        }
                    } else {
                        None
                    }
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

lla_plugin_interface::declare_plugin!(DuplicateFileDetectorPlugin);
