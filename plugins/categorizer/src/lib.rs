use colored::Colorize;
use dirs::config_dir;
use lla_plugin_interface::{DecoratedEntry, Plugin};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CategoryRule {
    name: String,
    color: String,
    extensions: Vec<String>,
    size_ranges: Option<Vec<(u64, u64)>>,
    subcategories: HashMap<String, Vec<String>>,
    description: String,
}

impl Default for CategoryRule {
    fn default() -> Self {
        Self {
            name: String::new(),
            color: "white".to_string(),
            extensions: Vec::new(),
            size_ranges: None,
            subcategories: HashMap::new(),
            description: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
struct CategoryStats {
    count: usize,
    total_size: u64,
    subcategory_counts: HashMap<String, usize>,
}


pub struct FileCategoryPlugin {
    rules: Vec<CategoryRule>,
    config_path: PathBuf,
    stats: HashMap<String, CategoryStats>,
}

impl FileCategoryPlugin {
    pub fn new() -> Self {
        let config_path = config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("lla")
            .join("categorizer.toml");

        let rules = Self::load_rules(&config_path).unwrap_or_else(|| {
            vec![
                CategoryRule {
                    name: "Document".to_string(),
                    color: "blue".to_string(),
                    extensions: vec!["txt", "md", "doc", "docx", "pdf", "rtf", "odt"]
                        .into_iter()
                        .map(String::from)
                        .collect(),
                    size_ranges: Some(vec![(0, 10_485_760)]),
                    subcategories: {
                        let mut map = HashMap::new();
                        map.insert(
                            "Text".to_string(),
                            vec!["txt", "md"].into_iter().map(String::from).collect(),
                        );
                        map.insert(
                            "Office".to_string(),
                            vec!["doc", "docx", "xls", "xlsx", "ppt", "pptx"]
                                .into_iter()
                                .map(String::from)
                                .collect(),
                        );
                        map
                    },
                    description: "Text documents and office files".to_string(),
                },
                CategoryRule {
                    name: "Image".to_string(),
                    color: "green".to_string(),
                    extensions: vec!["jpg", "jpeg", "png", "gif", "bmp", "svg", "webp", "tiff"]
                        .into_iter()
                        .map(String::from)
                        .collect(),
                    size_ranges: Some(vec![(0, 52_428_800)]),
                    subcategories: {
                        let mut map = HashMap::new();
                        map.insert(
                            "Raster".to_string(),
                            vec!["jpg", "jpeg", "png", "gif", "bmp"]
                                .into_iter()
                                .map(String::from)
                                .collect(),
                        );
                        map.insert(
                            "Vector".to_string(),
                            vec!["svg", "ai", "eps"]
                                .into_iter()
                                .map(String::from)
                                .collect(),
                        );
                        map
                    },
                    description: "Image files in various formats".to_string(),
                },
                CategoryRule {
                    name: "Code".to_string(),
                    color: "cyan".to_string(),
                    extensions: vec![
                        "rs", "py", "js", "ts", "java", "c", "cpp", "h", "hpp", "go", "rb", "php",
                        "cs", "swift", "kt",
                    ]
                    .into_iter()
                    .map(String::from)
                    .collect(),
                    size_ranges: Some(vec![(0, 1_048_576)]),
                    subcategories: {
                        let mut map = HashMap::new();
                        map.insert(
                            "Systems".to_string(),
                            vec!["rs", "c", "cpp", "h", "hpp"]
                                .into_iter()
                                .map(String::from)
                                .collect(),
                        );
                        map.insert(
                            "Web".to_string(),
                            vec!["js", "ts", "html", "css", "php"]
                                .into_iter()
                                .map(String::from)
                                .collect(),
                        );
                        map.insert(
                            "Scripts".to_string(),
                            vec!["py", "rb", "sh", "bash"]
                                .into_iter()
                                .map(String::from)
                                .collect(),
                        );
                        map
                    },
                    description: "Source code files".to_string(),
                },
            ]
        });

        FileCategoryPlugin {
            rules,
            config_path,
            stats: HashMap::new(),
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

    fn load_rules(path: &PathBuf) -> Option<Vec<CategoryRule>> {
        fs::read_to_string(path)
            .ok()
            .and_then(|content| toml::from_str(&content).ok())
    }

    fn save_rules(&self) {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).ok();
        }
        if let Ok(content) = toml::to_string_pretty(&self.rules) {
            fs::write(&self.config_path, content).ok();
        }
    }

    fn get_category_info(
        &self,
        entry: &DecoratedEntry,
    ) -> Option<(String, String, Option<String>)> {
        let extension = entry.path.extension()?.to_str()?.to_lowercase();
        let size = entry.metadata.size;

        for rule in &self.rules {
            if rule.extensions.iter().any(|ext| ext == &extension) {
                if let Some(ranges) = &rule.size_ranges {
                    if !ranges.iter().any(|(min, max)| size >= *min && size <= *max) {
                        continue;
                    }
                }

                let subcategory = rule
                    .subcategories
                    .iter()
                    .find(|(_, exts)| exts.contains(&extension))
                    .map(|(name, _)| name.clone());

                return Some((rule.name.clone(), rule.color.clone(), subcategory));
            }
        }
        None
    }

    fn update_stats(&mut self, entry: &DecoratedEntry, category: &str, subcategory: Option<&str>) {
        let stats = self.stats.entry(category.to_string()).or_default();
        stats.count += 1;
        stats.total_size += entry.metadata.size;
        if let Some(sub) = subcategory {
            *stats.subcategory_counts.entry(sub.to_string()).or_default() += 1;
        }
    }

    fn string_to_color(color: &str) -> colored::Color {
        match color.to_lowercase().as_str() {
            "black" => colored::Color::Black,
            "red" => colored::Color::Red,
            "green" => colored::Color::Green,
            "yellow" => colored::Color::Yellow,
            "blue" => colored::Color::Blue,
            "magenta" => colored::Color::Magenta,
            "cyan" => colored::Color::Cyan,
            _ => colored::Color::White,
        }
    }

    fn format_stats(&self) -> String {
        let mut output = String::new();
        output.push_str("Category Statistics:\n");

        for (category, stats) in &self.stats {
            let color = self
                .rules
                .iter()
                .find(|r| &r.name == category)
                .map(|r| Self::string_to_color(&r.color))
                .unwrap_or(colored::Color::White);

            output.push_str(&format!(
                "\n{} ({} files, {})\n",
                category.color(color),
                stats.count,
                Self::format_size(stats.total_size)
            ));

            for (sub, count) in &stats.subcategory_counts {
                output.push_str(&format!("  {} ({} files)\n", sub, count));
            }
        }
        output
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

impl Plugin for FileCategoryPlugin {
    fn handle_raw_request(&mut self, request: &[u8]) -> Vec<u8> {
        use lla_plugin_interface::proto::{self, plugin_message};
        use prost::Message as ProstMessage;

        let proto_msg = match proto::PluginMessage::decode(request) {
            Ok(msg) => msg,
            Err(e) => {
                let error_msg = proto::PluginMessage {
                    message: Some(plugin_message::Message::ErrorResponse(format!(
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
            Some(plugin_message::Message::GetName(_)) => {
                plugin_message::Message::NameResponse(env!("CARGO_PKG_NAME").to_string())
            }
            Some(plugin_message::Message::GetVersion(_)) => {
                plugin_message::Message::VersionResponse(env!("CARGO_PKG_VERSION").to_string())
            }
            Some(plugin_message::Message::GetDescription(_)) => {
                plugin_message::Message::DescriptionResponse(
                    env!("CARGO_PKG_DESCRIPTION").to_string(),
                )
            }
            Some(plugin_message::Message::GetSupportedFormats(_)) => {
                plugin_message::Message::FormatsResponse(proto::SupportedFormatsResponse {
                    formats: vec!["default".to_string(), "long".to_string()],
                })
            }
            Some(plugin_message::Message::Decorate(entry)) => {
                let mut decorated_entry = match DecoratedEntry::try_from(entry.clone()) {
                    Ok(e) => e,
                    Err(e) => {
                        return self.encode_error(&format!("Failed to convert entry: {}", e));
                    }
                };

                if let Some((category, color, subcategory)) =
                    self.get_category_info(&decorated_entry)
                {
                    decorated_entry
                        .custom_fields
                        .insert("category".to_string(), category.clone());
                    decorated_entry
                        .custom_fields
                        .insert("category_color".to_string(), color);
                    if let Some(sub) = &subcategory {
                        decorated_entry
                            .custom_fields
                            .insert("subcategory".to_string(), sub.clone());
                    }
                    self.update_stats(&decorated_entry, &category, subcategory.as_deref());
                }

                plugin_message::Message::DecoratedResponse(decorated_entry.into())
            }
            Some(plugin_message::Message::FormatField(req)) => {
                let entry = match req.entry {
                    Some(e) => match DecoratedEntry::try_from(e) {
                        Ok(entry) => entry,
                        Err(e) => {
                            return self.encode_error(&format!("Failed to convert entry: {}", e));
                        }
                    },
                    None => return self.encode_error("Missing entry in format field request"),
                };

                let formatted = match req.format.as_str() {
                    "default" => entry.custom_fields.get("category").map(|category| {
                        let color = entry
                            .custom_fields
                            .get("category_color")
                            .and_then(|c| c.parse::<colored::Color>().ok())
                            .unwrap_or(colored::Color::White);
                        format!("[{}]", category.color(color))
                    }),
                    "long" => entry.custom_fields.get("category").map(|category| {
                        let color = entry
                            .custom_fields
                            .get("category_color")
                            .and_then(|c| c.parse::<colored::Color>().ok())
                            .unwrap_or(colored::Color::White);
                        let base = format!("[{}]", category.color(color));
                        if let Some(sub) = entry.custom_fields.get("subcategory") {
                            format!("{} ({})", base, sub.bright_black())
                        } else {
                            base
                        }
                    }),
                    _ => None,
                };

                plugin_message::Message::FieldResponse(proto::FormattedFieldResponse {
                    field: formatted,
                })
            }
            Some(plugin_message::Message::Action(req)) => {
                let result: Result<(), String> = match req.action.as_str() {
                    "add-category" => {
                        if req.args.len() < 3 {
                            Err(
                                "Usage: add-category <n> <color> <ext1,ext2,...> [description]"
                                    .to_string(),
                            )
                        } else {
                            let mut rule = CategoryRule::default();
                            rule.name = req.args[0].clone();
                            rule.color = req.args[1].clone();
                            rule.extensions = req.args[2].split(',').map(String::from).collect();
                            if let Some(desc) = req.args.get(3) {
                                rule.description = desc.clone();
                            }
                            self.rules.push(rule);
                            self.save_rules();
                            Ok(())
                        }
                    }
                    "add-subcategory" => {
                        if req.args.len() != 4 {
                            Err(
                                "Usage: add-subcategory <category> <subcategory> <ext1,ext2,...>"
                                    .to_string(),
                            )
                        } else if let Some(rule) =
                            self.rules.iter_mut().find(|r| r.name == req.args[0])
                        {
                            rule.subcategories.insert(
                                req.args[1].clone(),
                                req.args[2].split(',').map(String::from).collect(),
                            );
                            self.save_rules();
                            Ok(())
                        } else {
                            Err(format!("Category '{}' not found", req.args[0]))
                        }
                    }
                    "show-stats" => {
                        println!("{}", self.format_stats());
                        Ok(())
                    }
                    "list-categories" => {
                        for rule in &self.rules {
                            let color = Self::string_to_color(&rule.color);
                            println!("\n{} ({})", rule.name.color(color), rule.description);
                            println!("  Extensions: {}", rule.extensions.join(", "));
                            if !rule.subcategories.is_empty() {
                                println!("  Subcategories:");
                                for (sub, exts) in &rule.subcategories {
                                    println!("    {}: {}", sub, exts.join(", "));
                                }
                            }
                        }
                        Ok(())
                    }
                    "help" => {
                        let help_text = "Available actions:\n\
                                add-category <n> <color> <ext1,ext2,...> [description] - Add a new category\n\
                                add-subcategory <category> <subcategory> <ext1,ext2,...> - Add a subcategory\n\
                                show-stats - Show category statistics\n\
                                list-categories - List all categories and their details\n\
                                help - Show this help message\n\n";
                        println!("{}", help_text);
                        Ok(())
                    }
                    _ => Err(format!("Unknown action: {}", req.action)),
                };

                plugin_message::Message::ActionResponse(proto::ActionResponse {
                    success: result.is_ok(),
                    error: result.err(),
                })
            }
            _ => plugin_message::Message::ErrorResponse("Invalid request type".to_string()),
        };

        let response = proto::PluginMessage {
            message: Some(response_msg),
        };
        let mut buf = bytes::BytesMut::with_capacity(response.encoded_len());
        response.encode(&mut buf).unwrap();
        buf.to_vec()
    }
}

impl Default for FileCategoryPlugin {
    fn default() -> Self {
        Self::new()
    }
}

lla_plugin_interface::declare_plugin!(FileCategoryPlugin);
