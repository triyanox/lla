use colored::Colorize;
use lla_plugin_interface::{
    proto::{self, plugin_message::Message},
    Plugin,
};
use prost::Message as _;
use regex::RegexBuilder;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use toml;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct KeywordMatch {
    keyword: String,
    line_number: usize,
    line: String,
    context_before: Vec<String>,
    context_after: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SearchConfig {
    keywords: Vec<String>,
    case_sensitive: bool,
    use_regex: bool,
    context_lines: usize,
    max_matches: usize,
    file_extensions: Vec<String>,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            keywords: Vec::new(),
            case_sensitive: false,
            use_regex: false,
            context_lines: 2,
            max_matches: 5,
            file_extensions: vec![
                "txt".to_string(),
                "md".to_string(),
                "rs".to_string(),
                "py".to_string(),
                "js".to_string(),
                "java".to_string(),
                "c".to_string(),
                "cpp".to_string(),
                "h".to_string(),
                "hpp".to_string(),
                "go".to_string(),
                "rb".to_string(),
                "php".to_string(),
                "html".to_string(),
                "css".to_string(),
                "json".to_string(),
                "yaml".to_string(),
                "yml".to_string(),
                "toml".to_string(),
                "ini".to_string(),
                "conf".to_string(),
            ],
        }
    }
}

pub struct KeywordSearchPlugin {
    config: Arc<Mutex<SearchConfig>>,
    config_path: PathBuf,
}

impl KeywordSearchPlugin {
    pub fn new() -> Self {
        let config_path = dirs::home_dir()
            .expect("Failed to get home directory")
            .join(".config")
            .join("lla")
            .join("plugins")
            .join("keyword_search.toml");

        if let Some(parent) = config_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!(
                    "{} Failed to create plugin directory: {}",
                    "Warning:".bright_yellow(),
                    e
                );
            }
        }

        let config = Arc::new(Mutex::new(Self::load_config(&config_path)));
        KeywordSearchPlugin {
            config,
            config_path,
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

    fn load_config(path: &PathBuf) -> SearchConfig {
        match fs::read_to_string(path) {
            Ok(contents) => match toml::from_str(&contents) {
                Ok(config) => config,
                Err(e) => {
                    eprintln!(
                        "{} Failed to parse config: {}",
                        "Warning:".bright_yellow(),
                        e
                    );
                    if let Err(e) = fs::rename(path, path.with_extension("toml.bak")) {
                        eprintln!(
                            "{} Failed to backup corrupted config: {}",
                            "Warning:".bright_yellow(),
                            e
                        );
                    }
                    let default = SearchConfig::default();
                    if let Ok(contents) = toml::to_string_pretty(&default) {
                        if let Err(e) = fs::write(path, contents) {
                            eprintln!(
                                "{} Failed to write default config: {}",
                                "Warning:".bright_yellow(),
                                e
                            );
                        } else {
                            println!("{} Created new default config", "Info:".bright_blue());
                        }
                    }
                    default
                }
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let default = SearchConfig::default();
                if let Some(parent) = path.parent() {
                    if let Err(e) = fs::create_dir_all(parent) {
                        eprintln!(
                            "{} Failed to create config directory: {}",
                            "Warning:".bright_yellow(),
                            e
                        );
                        return default;
                    }
                }
                if let Ok(contents) = toml::to_string_pretty(&default) {
                    match fs::write(path, contents) {
                        Ok(_) => println!("{} Created new default config", "Info:".bright_blue()),
                        Err(e) => eprintln!(
                            "{} Failed to write default config: {}",
                            "Warning:".bright_yellow(),
                            e
                        ),
                    }
                }
                default
            }
            Err(e) => {
                eprintln!(
                    "{} Failed to read config: {}",
                    "Warning:".bright_yellow(),
                    e
                );
                SearchConfig::default()
            }
        }
    }

    fn save_config(&self) -> Result<(), String> {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        let config = self
            .config
            .lock()
            .map_err(|_| "Failed to acquire config lock".to_string())?;

        let temp_path = self.config_path.with_extension("toml.tmp");
        let contents = toml::to_string_pretty(&*config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(&temp_path, contents)
            .map_err(|e| format!("Failed to write temporary config: {}", e))?;

        match fs::rename(&temp_path, &self.config_path) {
            Ok(_) => {
                println!(
                    "{} Config saved to: {}",
                    "Info:".bright_blue(),
                    self.config_path.display().to_string().bright_yellow()
                );
                Ok(())
            }
            Err(e) => {
                let _ = fs::remove_file(&temp_path);
                Err(format!("Failed to save config: {}", e))
            }
        }
    }

    fn search_file(&self, path: &std::path::Path) -> Option<Vec<KeywordMatch>> {
        let config = self.config.lock().unwrap();

        if let Some(ext) = path.extension() {
            if !config
                .file_extensions
                .contains(&ext.to_string_lossy().to_string())
            {
                return None;
            }
        }

        let file = File::open(path).ok()?;
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader.lines().map_while(Result::ok).collect();

        let mut matches = Vec::new();
        let patterns: Vec<_> = config
            .keywords
            .iter()
            .map(|k| {
                RegexBuilder::new(&regex::escape(k))
                    .case_insensitive(!config.case_sensitive)
                    .build()
                    .ok()
            })
            .collect();

        for (index, line) in lines.iter().enumerate() {
            for (pattern_index, pattern) in patterns.iter().enumerate() {
                if let Some(pattern) = pattern {
                    if pattern.is_match(line) {
                        let context_start = index.saturating_sub(config.context_lines);
                        let context_end = (index + config.context_lines + 1).min(lines.len());

                        matches.push(KeywordMatch {
                            keyword: config.keywords[pattern_index].clone(),
                            line_number: index + 1,
                            line: line.clone(),
                            context_before: lines[context_start..index].to_vec(),
                            context_after: lines[index + 1..context_end].to_vec(),
                        });

                        if matches.len() >= config.max_matches {
                            return Some(matches);
                        }
                    }
                }
            }
        }

        if matches.is_empty() {
            None
        } else {
            Some(matches)
        }
    }

    fn format_matches(&self, matches: &[KeywordMatch], long: bool) -> String {
        let mut output = String::new();
        for m in matches {
            if long {
                for (i, line) in m.context_before.iter().enumerate() {
                    let line_num = m.line_number - (m.context_before.len() - i);
                    output.push_str(&format!(
                        "  {}: {}\n",
                        line_num.to_string().bright_black(),
                        line
                    ));
                }

                output.push_str(&format!(
                    "→ {}: {}\n",
                    m.line_number.to_string().bright_yellow(),
                    m.line
                        .replace(&m.keyword, &m.keyword.bright_red().to_string())
                ));

                for (i, line) in m.context_after.iter().enumerate() {
                    let line_num = m.line_number + i + 1;
                    output.push_str(&format!(
                        "  {}: {}\n",
                        line_num.to_string().bright_black(),
                        line
                    ));
                }
                output.push('\n');
            } else {
                output.push_str(&format!(
                    "{}:{} - {}\n",
                    m.line_number,
                    m.keyword.bright_red(),
                    m.line.trim()
                ));
            }
        }
        output
    }
}

impl Plugin for KeywordSearchPlugin {
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

                if let Some(matches) = entry
                    .path
                    .is_file()
                    .then(|| self.search_file(&entry.path))
                    .flatten()
                {
                    entry.custom_fields.insert(
                        "keyword_matches".to_string(),
                        toml::to_string(&matches).unwrap_or_default(),
                    );
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

                let formatted = entry
                    .custom_fields
                    .get("keyword_matches")
                    .and_then(|toml_str| toml::from_str::<Vec<KeywordMatch>>(toml_str).ok())
                    .map(|matches| self.format_matches(&matches[..], req.format == "long"));
                Message::FieldResponse(proto::FormattedFieldResponse { field: formatted })
            }
            Some(Message::Action(req)) => match req.action.as_str() {
                "set-keywords" => {
                    if req.args.is_empty() {
                        return self.encode_error("Usage: set-keywords <keyword1> [keyword2 ...]");
                    }
                    let mut config = self.config.lock().unwrap();
                    config.keywords = req.args.to_vec();
                    drop(config);
                    if let Err(e) = self.save_config() {
                        return self.encode_error(&e);
                    }
                    println!(
                        "Set keywords to: {}",
                        req.args
                            .iter()
                            .map(|k| k.yellow().to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                    Message::ActionResponse(proto::ActionResponse {
                        success: true,
                        error: None,
                    })
                }
                "show-config" => {
                    let config = self.config.lock().unwrap();
                    println!("Current configuration:");
                    println!(
                        "  Keywords: {}",
                        config
                            .keywords
                            .iter()
                            .map(|k| k.yellow().to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                    println!(
                        "  Case sensitive: {}",
                        config.case_sensitive.to_string().cyan()
                    );
                    println!("  Use regex: {}", config.use_regex.to_string().cyan());
                    println!(
                        "  Context lines: {}",
                        config.context_lines.to_string().cyan()
                    );
                    println!(
                        "  Max matches per file: {}",
                        config.max_matches.to_string().cyan()
                    );
                    println!(
                        "  File extensions: {}",
                        config
                            .file_extensions
                            .iter()
                            .map(|e| e.cyan().to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                    Message::ActionResponse(proto::ActionResponse {
                        success: true,
                        error: None,
                    })
                }
                "set-case-sensitive" => {
                    let value = req.args.first().map(|s| s == "true").unwrap_or(false);
                    let mut config = self.config.lock().unwrap();
                    config.case_sensitive = value;
                    drop(config);
                    if let Err(e) = self.save_config() {
                        return self.encode_error(&e);
                    }
                    println!("Case sensitive search: {}", value.to_string().cyan());
                    Message::ActionResponse(proto::ActionResponse {
                        success: true,
                        error: None,
                    })
                }
                "set-context-lines" => {
                    if let Some(lines) = req.args.first().and_then(|s| s.parse().ok()) {
                        let mut config = self.config.lock().unwrap();
                        config.context_lines = lines;
                        drop(config);
                        if let Err(e) = self.save_config() {
                            return self.encode_error(&e);
                        }
                        println!("Context lines set to: {}", lines.to_string().cyan());
                        Message::ActionResponse(proto::ActionResponse {
                            success: true,
                            error: None,
                        })
                    } else {
                        return self.encode_error("Invalid number of context lines");
                    }
                }
                "set-max-matches" => {
                    if let Some(max) = req.args.first().and_then(|s| s.parse().ok()) {
                        let mut config = self.config.lock().unwrap();
                        config.max_matches = max;
                        drop(config);
                        if let Err(e) = self.save_config() {
                            return self.encode_error(&e);
                        }
                        println!("Max matches per file set to: {}", max.to_string().cyan());
                        Message::ActionResponse(proto::ActionResponse {
                            success: true,
                            error: None,
                        })
                    } else {
                        return self.encode_error("Invalid max matches value");
                    }
                }
                "help" => {
                    println!("{}", "Keyword Search Plugin".bright_green().bold());
                    println!("");
                    println!("{}", "Actions:".bright_yellow());
                    println!(
                        "  {} <keyword1> [keyword2 ...]",
                        "set-keywords".bright_cyan()
                    );
                    println!("    Set keywords to search for in files");
                    println!("");
                    println!("  {}", "show-config".bright_cyan());
                    println!("    Display current plugin configuration");
                    println!("");
                    println!("  {} [true|false]", "set-case-sensitive".bright_cyan());
                    println!("    Enable or disable case-sensitive search");
                    println!("");
                    println!("  {} <number>", "set-context-lines".bright_cyan());
                    println!("    Set number of context lines to show around matches");
                    println!("");
                    println!("  {} <number>", "set-max-matches".bright_cyan());
                    println!("    Set maximum number of matches to show per file");
                    println!("");
                    println!("  {} <file_path>", "search".bright_cyan());
                    println!("    Search for configured keywords in a specific file");
                    println!("");
                    println!("  Configure search:");
                    println!(
                        "    {} --name keyword_search --action set-context-lines --args 3",
                        "lla plugin".bright_blue()
                    );
                    println!(
                        "    {} --name keyword_search --action set-case-sensitive --args true",
                        "lla plugin".bright_blue()
                    );
                    Message::ActionResponse(proto::ActionResponse {
                        success: true,
                        error: None,
                    })
                }
                "search" => {
                    if req.args.is_empty() {
                        return self.encode_error("Usage: search <file_path>");
                    }
                    let path = std::path::Path::new(&req.args[0]);
                    if let Some(matches) = self.search_file(path) {
                        println!(
                            "\n{} in {}:",
                            "Matches".bright_green().bold(),
                            path.display().to_string().bright_blue()
                        );
                        println!("{}", self.format_matches(&matches[..], true));
                        Message::ActionResponse(proto::ActionResponse {
                            success: true,
                            error: None,
                        })
                    } else {
                        println!(
                            "{} No matches found in {}",
                            "Info:".bright_blue(),
                            path.display().to_string().bright_yellow()
                        );
                        Message::ActionResponse(proto::ActionResponse {
                            success: true,
                            error: None,
                        })
                    }
                }
                _ => {
                    return self.encode_error(&format!("Unknown action: {}", req.action));
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

impl Default for KeywordSearchPlugin {
    fn default() -> Self {
        Self::new()
    }
}

lla_plugin_interface::declare_plugin!(KeywordSearchPlugin);
