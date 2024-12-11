use colored::Colorize;
use lla_plugin_interface::{
    proto::{self, plugin_message::Message},
    Plugin,
};
use prost::Message as _;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

pub struct FileTaggerPlugin {
    tag_file: PathBuf,
    tags: HashMap<String, Vec<String>>,
}

impl Default for FileTaggerPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl FileTaggerPlugin {
    pub fn new() -> Self {
        let tag_file = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("lla")
            .join("file_tags.txt");
        let tags = Self::load_tags(&tag_file);
        FileTaggerPlugin { tag_file, tags }
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

    fn load_tags(path: &PathBuf) -> HashMap<String, Vec<String>> {
        let mut tags: HashMap<String, Vec<String>> = HashMap::new();
        if let Ok(file) = File::open(path) {
            let reader = BufReader::new(file);
            for line in reader.lines().map_while(Result::ok) {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() == 2 {
                    tags.entry(parts[0].to_string())
                        .or_default()
                        .push(parts[1].to_string());
                }
            }
        }
        tags
    }

    fn save_tags(&self) {
        if let Some(parent) = self.tag_file.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        if let Ok(mut file) = File::create(&self.tag_file) {
            for (file_path, tags) in &self.tags {
                for tag in tags {
                    writeln!(file, "{}|{}", file_path, tag).ok();
                }
            }
        }
    }

    fn add_tag(&mut self, file_path: &str, tag: &str) {
        self.tags
            .entry(file_path.to_string())
            .or_default()
            .push(tag.to_string());
        self.save_tags();
    }

    fn remove_tag(&mut self, file_path: &str, tag: &str) {
        if let Some(tags) = self.tags.get_mut(file_path) {
            tags.retain(|t| t != tag);
            if tags.is_empty() {
                self.tags.remove(file_path);
            }
        }
        self.save_tags();
    }

    fn get_tags(&self, file_path: &str) -> Vec<String> {
        self.tags.get(file_path).cloned().unwrap_or_default()
    }
}

impl Plugin for FileTaggerPlugin {
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
                    formats: vec!["default".to_string()],
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

                let tags = self.get_tags(entry.path.to_str().unwrap_or(""));
                if !tags.is_empty() {
                    entry
                        .custom_fields
                        .insert("tags".to_string(), tags.join(", "));
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

                let formatted = entry.custom_fields.get("tags").map(|tags| {
                    format!(
                        "[{}]",
                        tags.split(", ")
                            .map(|t| t.cyan().to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                });
                Message::FieldResponse(proto::FormattedFieldResponse { field: formatted })
            }
            Some(Message::Action(req)) => match req.action.as_str() {
                "add-tag" => {
                    if req.args.len() != 2 {
                        println!("{} add-tag <file_path> <tag>", "Usage:".bright_cyan());
                        return self.encode_error("Invalid number of arguments for add-tag");
                    }
                    self.add_tag(&req.args[0], &req.args[1]);
                    println!(
                        "{} tag '{}' to {}",
                        "Added".bright_green(),
                        req.args[1].cyan(),
                        req.args[0].bright_blue()
                    );
                    Message::ActionResponse(proto::ActionResponse {
                        success: true,
                        error: None,
                    })
                }
                "remove-tag" => {
                    if req.args.len() != 2 {
                        println!("{} remove-tag <file_path> <tag>", "Usage:".bright_cyan());
                        return self.encode_error("Invalid number of arguments for remove-tag");
                    }
                    self.remove_tag(&req.args[0], &req.args[1]);
                    println!(
                        "{} tag '{}' from {}",
                        "Removed".bright_green(),
                        req.args[1].cyan(),
                        req.args[0].bright_blue()
                    );
                    Message::ActionResponse(proto::ActionResponse {
                        success: true,
                        error: None,
                    })
                }
                "list-tags" => {
                    if req.args.len() != 1 {
                        println!("{} list-tags <file_path>", "Usage:".bright_cyan());
                        return self.encode_error("Invalid number of arguments for list-tags");
                    }
                    let tags = self.get_tags(&req.args[0]);
                    if tags.is_empty() {
                        println!(
                            "{} No tags found for {}",
                            "Info:".bright_blue(),
                            req.args[0].bright_yellow()
                        );
                    } else {
                        println!(
                            "{} for {}:",
                            "Tags".bright_green(),
                            req.args[0].bright_blue()
                        );
                        for tag in tags {
                            println!("  {} {}", "→".bright_cyan(), tag.bright_yellow());
                        }
                    }
                    Message::ActionResponse(proto::ActionResponse {
                        success: true,
                        error: None,
                    })
                }
                "help" => {
                    println!("{}", "File Tagger Commands".bright_green().bold());
                    println!("");
                    println!("{}", "Available actions:".bright_yellow());
                    println!("  {} <file_path> <tag>", "add-tag".bright_cyan());
                    println!("    Add a tag to a file");
                    println!("");
                    println!("  {} <file_path> <tag>", "remove-tag".bright_cyan());
                    println!("    Remove a tag from a file");
                    println!("");
                    println!("  {} <file_path>", "list-tags".bright_cyan());
                    println!("    List all tags for a file");
                    Message::ActionResponse(proto::ActionResponse {
                        success: true,
                        error: None,
                    })
                }
                _ => {
                    println!("{} Unknown action: {}", "Error:".bright_red(), req.action);
                    Message::ActionResponse(proto::ActionResponse {
                        success: false,
                        error: Some(format!("Unknown action: {}", req.action)),
                    })
                }
            },
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

lla_plugin_interface::declare_plugin!(FileTaggerPlugin);
