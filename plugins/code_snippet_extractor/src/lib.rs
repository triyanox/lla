use base64::Engine as _;
use colored::Colorize;
use lla_plugin_interface::{DecoratedEntry, Plugin};
use ring::digest;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use toml;

#[derive(Clone, Serialize, Deserialize)]
struct CodeSnippet {
    name: String,
    content: String,
    language: String,
    tags: HashSet<String>,
    created_at: u64,
    modified_at: u64,
    version: u32,
    context_before: Option<String>,
    context_after: Option<String>,
    hash: String,
}

impl CodeSnippet {
    fn new(name: String, content: String, language: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let hash = Self::compute_hash(&content);

        Self {
            name,
            content,
            language,
            tags: HashSet::new(),
            created_at: timestamp,
            modified_at: timestamp,
            version: 1,
            context_before: None,
            context_after: None,
            hash,
        }
    }

    fn compute_hash(content: &str) -> String {
        let hash = digest::digest(&digest::SHA256, content.as_bytes());
        base64::engine::general_purpose::STANDARD.encode(hash.as_ref())
    }
}

#[derive(Clone)]
pub struct CodeSnippetExtractorPlugin {
    snippet_file: PathBuf,
    snippets: HashMap<String, Vec<CodeSnippet>>,
}

impl CodeSnippetExtractorPlugin {
    pub fn new() -> Self {
        let snippet_file = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("lla")
            .join("code_snippets.toml");
        let snippets = Self::load_snippets(&snippet_file);
        CodeSnippetExtractorPlugin {
            snippet_file,
            snippets,
        }
    }

    fn load_snippets(path: &PathBuf) -> HashMap<String, Vec<CodeSnippet>> {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(snippets) = toml::from_str(&content) {
                return snippets;
            }
        }
        HashMap::new()
    }

    fn save_snippets(&self) -> Result<(), String> {
        if let Some(parent) = self.snippet_file.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let content = toml::to_string_pretty(&self.snippets)
            .map_err(|e| format!("Failed to serialize snippets: {}", e))?;
        std::fs::write(&self.snippet_file, content).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn detect_language(file_path: &str) -> String {
        match file_path.split('.').last() {
            Some("rs") => "rust",
            Some("py") => "python",
            Some("js") => "javascript",
            Some("ts") => "typescript",
            Some("go") => "go",
            Some("c") => "c",
            Some("cpp") | Some("cc") | Some("cxx") => "cpp",
            Some("java") => "java",
            Some("rb") => "ruby",
            Some("php") => "php",
            Some("sh") => "shell",
            Some("html") => "html",
            Some("css") => "css",
            Some("md") => "markdown",
            Some("json") => "json",
            Some("yaml") | Some("yml") => "yaml",
            Some("xml") => "xml",
            Some("sql") => "sql",
            _ => "text",
        }
        .to_string()
    }

    fn extract_snippet(
        &mut self,
        file_path: &str,
        name: &str,
        start_line: usize,
        end_line: usize,
        context_lines: Option<usize>,
    ) -> Result<(), String> {
        let file = File::open(file_path).map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader.lines().map_while(Result::ok).collect();

        if start_line > end_line || end_line > lines.len() {
            return Err("Invalid line range".to_string());
        }

        let context_lines = context_lines.unwrap_or(3);
        let context_before = if start_line > context_lines {
            Some(lines[start_line - context_lines - 1..start_line - 1].join("\n"))
        } else {
            Some(lines[0..start_line - 1].join("\n"))
        };

        let context_after = if end_line + context_lines <= lines.len() {
            Some(lines[end_line..end_line + context_lines].join("\n"))
        } else {
            Some(lines[end_line..].join("\n"))
        };

        let content = lines[start_line - 1..end_line].join("\n");
        let language = Self::detect_language(file_path);
        let mut snippet = CodeSnippet::new(name.to_string(), content, language);
        snippet.context_before = context_before;
        snippet.context_after = context_after;

        self.snippets
            .entry(file_path.to_string())
            .or_default()
            .push(snippet);

        self.save_snippets()?;
        Ok(())
    }

    fn list_snippets(&self, file_path: &str) -> Vec<String> {
        self.snippets
            .get(file_path)
            .map(|snippets| {
                snippets
                    .iter()
                    .map(|s| {
                        format!(
                            "{} [v{}] [{}] {}",
                            s.name,
                            s.version,
                            s.language,
                            s.tags
                                .iter()
                                .map(|t| format!("#{}", t))
                                .collect::<Vec<_>>()
                                .join(" ")
                        )
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn get_snippet(&self, file_path: &str, name: &str) -> Option<&CodeSnippet> {
        self.snippets
            .get(file_path)?
            .iter()
            .find(|s| s.name == name)
    }

    fn search_snippets(&self, query: &str) -> Vec<(String, &CodeSnippet)> {
        let query = query.to_lowercase();
        let mut results = Vec::new();

        for (file_path, snippets) in &self.snippets {
            for snippet in snippets {
                if snippet.name.to_lowercase().contains(&query)
                    || snippet.content.to_lowercase().contains(&query)
                    || snippet
                        .tags
                        .iter()
                        .any(|t| t.to_lowercase().contains(&query))
                {
                    results.push((file_path.clone(), snippet));
                }
            }
        }

        results
    }

    fn add_tags(&mut self, file_path: &str, name: &str, tags: &[String]) -> Result<(), String> {
        let snippets = self.snippets.get_mut(file_path).ok_or("File not found")?;
        let snippet = snippets
            .iter_mut()
            .find(|s| s.name == name)
            .ok_or("Snippet not found")?;

        snippet.tags.extend(tags.iter().cloned());
        self.save_snippets()?;
        Ok(())
    }

    fn remove_tags(&mut self, file_path: &str, name: &str, tags: &[String]) -> Result<(), String> {
        let snippets = self.snippets.get_mut(file_path).ok_or("File not found")?;
        let snippet = snippets
            .iter_mut()
            .find(|s| s.name == name)
            .ok_or("Snippet not found")?;

        for tag in tags {
            snippet.tags.remove(tag);
        }
        self.save_snippets()?;
        Ok(())
    }

    fn export_snippets(&self, file_path: &str) -> Result<String, String> {
        let snippets = self.snippets.get(file_path).ok_or("File not found")?;
        toml::to_string_pretty(snippets).map_err(|e| e.to_string())
    }

    fn import_snippets(&mut self, file_path: &str, toml_data: &str) -> Result<(), String> {
        let imported: Vec<CodeSnippet> =
            toml::from_str(toml_data).map_err(|e| format!("Invalid TOML format: {}", e))?;

        self.snippets.insert(file_path.to_string(), imported);
        self.save_snippets()?;
        Ok(())
    }
}

impl Plugin for CodeSnippetExtractorPlugin {
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

        let response_msg: plugin_message::Message = match proto_msg.message {
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
            Some(plugin_message::Message::Action(req)) => {
                let result: Result<(), String> = match req.action.as_str() {
                    "extract" => {
                        if req.args.len() < 4 || req.args.len() > 5 {
                            return self.encode_error("{} extract <file_path> <snippet_name> <start_line> <end_line> [context_lines]");
                        }
                        let start_line = match req.args[2].parse() {
                            Ok(n) => n,
                            Err(_) => {
                                return self.encode_error(
                                    format!("{} Invalid start line", "Error:".bright_red())
                                        .as_str(),
                                );
                            }
                        };
                        let end_line = match req.args[3].parse() {
                            Ok(n) => n,
                            Err(_) => {
                                return self.encode_error(
                                    format!("{} Invalid end line", "Error:".bright_red()).as_str(),
                                );
                            }
                        };
                        let context_lines = req.args.get(4).and_then(|s| s.parse().ok());

                        match self.extract_snippet(
                            &req.args[0],
                            &req.args[1],
                            start_line,
                            end_line,
                            context_lines,
                        ) {
                            Ok(()) => {
                                println!(
                                    "{} extracted snippet '{}' from {} (lines {}-{})",
                                    "Successfully".bright_green(),
                                    req.args[1].bright_yellow(),
                                    req.args[0].bright_blue(),
                                    start_line.to_string().bright_cyan(),
                                    end_line.to_string().bright_cyan()
                                );
                                Ok(())
                            }
                            Err(e) => {
                                println!("{} {}", "Error:".bright_red(), e);
                                Ok(())
                            }
                        }
                    }
                    "list" => {
                        if req.args.len() != 1 {
                            return self.encode_error(
                                format!("{} list <file_path>", "Usage:".bright_cyan()).as_str(),
                            );
                        }
                        let snippets = self.list_snippets(&req.args[0]);
                        if snippets.is_empty() {
                            println!(
                                "{} No snippets found in {}",
                                "Info:".bright_blue(),
                                req.args[0].bright_yellow()
                            );
                        } else {
                            println!(
                                "{} in {}:",
                                "Snippets".bright_green(),
                                req.args[0].bright_blue()
                            );
                            for snippet in snippets {
                                println!("  {}", snippet);
                            }
                        }
                        Ok(())
                    }
                    "get" => {
                        if req.args.len() != 2 {
                            return self.encode_error(
                                format!(
                                    "{} get <file_path> <snippet_name>",
                                    "Usage:".bright_cyan()
                                )
                                .as_str(),
                            );
                        }
                        match self.get_snippet(&req.args[0], &req.args[1]) {
                            Some(snippet) => {
                                println!("{}", "┌─ Context Before ─────────────────".bright_cyan());
                                if let Some(ctx) = &snippet.context_before {
                                    println!("{}", ctx.dimmed());
                                }
                                println!("{}", "├─ Snippet Content ───────────────".bright_green());
                                println!("{}", snippet.content.bright_white());
                                println!("{}", "├─ Context After ──────────────────".bright_cyan());
                                if let Some(ctx) = &snippet.context_after {
                                    println!("{}", ctx.dimmed());
                                }
                                println!("{}", "├─ Metadata ─────────────────────".bright_yellow());
                                println!("│ {}: {}", "Language".bright_blue(), snippet.language);
                                println!("│ {}: {}", "Version".bright_blue(), snippet.version);
                                println!(
                                    "│ {}: {}",
                                    "Tags".bright_blue(),
                                    snippet
                                        .tags
                                        .iter()
                                        .map(|t| format!("#{}", t.bright_magenta()))
                                        .collect::<Vec<_>>()
                                        .join(" ")
                                );
                                println!("{}", "└────────────────────────────────".bright_cyan());
                                Ok(())
                            }
                            None => {
                                println!(
                                    "{} Snippet '{}' not found in {}",
                                    "Error:".bright_red(),
                                    req.args[1].bright_yellow(),
                                    req.args[0].bright_blue()
                                );
                                Ok(())
                            }
                        }
                    }
                    "search" => {
                        if req.args.len() != 1 {
                            return self.encode_error(
                                format!("{} search <query>", "Usage:".bright_cyan()).as_str(),
                            );
                        }
                        let results = self.search_snippets(&req.args[0]);
                        if results.is_empty() {
                            println!(
                                "{} No matching snippets found for query: {}",
                                "Info:".bright_blue(),
                                req.args[0].bright_yellow()
                            );
                        } else {
                            println!(
                                "{} snippets for query: {}",
                                "Found".bright_green(),
                                req.args[0].bright_yellow()
                            );
                            for (file, snippet) in results {
                                println!(
                                    "  {} {} [{}] {}",
                                    "→".bright_cyan(),
                                    file.bright_blue(),
                                    snippet.name.bright_yellow(),
                                    snippet
                                        .tags
                                        .iter()
                                        .map(|t| format!("#{}", t.bright_magenta()))
                                        .collect::<Vec<_>>()
                                        .join(" ")
                                );
                            }
                        }
                        Ok(())
                    }
                    "add-tags" => {
                        if req.args.len() < 3 {
                            return self.encode_error(
                                format!(
                                    "{} add-tags <file_path> <snippet_name> <tag1> [tag2...]",
                                    "Usage:".bright_cyan()
                                )
                                .as_str(),
                            );
                        }
                        let tags: Vec<String> =
                            req.args[2..].iter().map(|s| s.to_string()).collect();
                        match self.add_tags(&req.args[0], &req.args[1], &tags) {
                            Ok(()) => {
                                println!(
                                    "{} tags {} to snippet '{}'",
                                    "Added".bright_green(),
                                    tags.iter()
                                        .map(|t| format!("#{}", t.bright_magenta()))
                                        .collect::<Vec<_>>()
                                        .join(" "),
                                    req.args[1].bright_yellow()
                                );
                                Ok(())
                            }
                            Err(e) => {
                                println!("{} {}", "Error:".bright_red(), e);
                                Ok(())
                            }
                        }
                    }
                    "remove-tags" => {
                        if req.args.len() < 3 {
                            return self.encode_error(
                                format!(
                                    "{} remove-tags <file_path> <snippet_name> <tag1> [tag2...]",
                                    "Usage:".bright_cyan()
                                )
                                .as_str(),
                            );
                        }
                        let tags: Vec<String> =
                            req.args[2..].iter().map(|s| s.to_string()).collect();
                        match self.remove_tags(&req.args[0], &req.args[1], &tags) {
                            Ok(()) => {
                                println!(
                                    "{} tags {} from snippet '{}'",
                                    "Removed".bright_green(),
                                    tags.iter()
                                        .map(|t| format!("#{}", t.bright_magenta()))
                                        .collect::<Vec<_>>()
                                        .join(" "),
                                    req.args[1].bright_yellow()
                                );
                                Ok(())
                            }
                            Err(e) => {
                                println!("{} {}", "Error:".bright_red(), e);
                                Ok(())
                            }
                        }
                    }
                    "export" => {
                        if req.args.len() != 1 {
                            return self.encode_error(
                                format!("{} export <file_path>", "Usage:".bright_cyan()).as_str(),
                            );
                        }
                        match self.export_snippets(&req.args[0]) {
                            Ok(toml) => {
                                println!(
                                    "{} Exported snippets from {}",
                                    "Successfully".bright_green(),
                                    req.args[0].bright_blue()
                                );
                                println!("{}", toml);
                                Ok(())
                            }
                            Err(e) => {
                                println!("{} {}", "Error:".bright_red(), e);
                                Ok(())
                            }
                        }
                    }
                    "import" => {
                        if req.args.len() != 2 {
                            return self.encode_error(
                                format!(
                                    "{} import <file_path> <toml_data>",
                                    "Usage:".bright_cyan()
                                )
                                .as_str(),
                            );
                        }
                        match self.import_snippets(&req.args[0], &req.args[1]) {
                            Ok(()) => {
                                println!(
                                    "{} imported snippets to {}",
                                    "Successfully".bright_green(),
                                    req.args[0].bright_blue()
                                );
                                Ok(())
                            }
                            Err(e) => {
                                println!("{} {}", "Error:".bright_red(), e);
                                Ok(())
                            }
                        }
                    }
                    "help" => {
                        println!(
                            "{}",
                            "Code Snippet Extractor Commands".bright_green().bold()
                        );
                        println!("");
                        println!("{}", "Basic Commands:".bright_yellow());
                        println!("  {} <file_path> <snippet_name> <start_line> <end_line> [context_lines]", "extract".bright_cyan());
                        println!("    Extract a code snippet from a file");
                        println!("");
                        println!("  {} <file_path>", "list".bright_cyan());
                        println!("    List all snippets in a file");
                        println!("");
                        println!("  {} <file_path> <snippet_name>", "get".bright_cyan());
                        println!("    Get a specific snippet with context");
                        println!("");
                        println!("{}", "Search & Organization:".bright_yellow());
                        println!("  {} <query>", "search".bright_cyan());
                        println!("    Search through all snippets");
                        println!("");
                        println!(
                            "  {} <file_path> <snippet_name> <tag1> [tag2...]",
                            "add-tags".bright_cyan()
                        );
                        println!("    Add tags to a snippet");
                        println!("");
                        println!(
                            "  {} <file_path> <snippet_name> <tag1> [tag2...]",
                            "remove-tags".bright_cyan()
                        );
                        println!("    Remove tags from a snippet");
                        println!("");
                        println!("{}", "Import/Export:".bright_yellow());
                        println!("  {} <file_path>", "export".bright_cyan());
                        println!("    Export snippets to TOML format");
                        println!("");
                        println!("  {} <file_path> <toml_data>", "import".bright_cyan());
                        println!("    Import snippets from TOML");
                        println!("");
                        println!("{}", "Examples:".bright_yellow());
                        println!("  {} Extract lines 10-20 from a file:", "→".bright_cyan());
                        println!("    lla plugin --name code_snippet_extractor --action extract --args \"file.rs\" \"my_func\" 10 20");
                        println!("");
                        println!("  {} Add tags to a snippet:", "→".bright_cyan());
                        println!("    lla plugin --name code_snippet_extractor --action add-tags --args \"file.rs\" \"my_func\" \"rust\" \"function\"");
                        Ok(())
                    }
                    _ => {
                        println!("{} Unknown action: {}", "Error:".bright_red(), req.action);
                        Ok(())
                    }
                };
                plugin_message::Message::ActionResponse(proto::ActionResponse {
                    success: result.is_ok(),
                    error: result.err(),
                })
            }
            Some(plugin_message::Message::Decorate(entry)) => {
                let mut entry = match DecoratedEntry::try_from(entry.clone()) {
                    Ok(e) => e,
                    Err(e) => {
                        return self.encode_error(&format!("Failed to convert entry: {}", e));
                    }
                };
                if let Some(file_path) = entry.path.to_str() {
                    let snippet_count = self.snippets.get(file_path).map(|s| s.len()).unwrap_or(0);
                    if snippet_count > 0 {
                        entry.custom_fields.insert(
                            "snippet_count".to_string(),
                            format!("[{} snippets]", snippet_count.to_string().bright_yellow()),
                        );
                    }
                }
                plugin_message::Message::DecoratedResponse(entry.into())
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

                if req.format == "snippet_count" {
                    if let Some(count) = entry.custom_fields.get("snippet_count") {
                        plugin_message::Message::FieldResponse(proto::FormattedFieldResponse {
                            field: Some(format!("[{} snippets]", count.bright_yellow())),
                        })
                    } else {
                        plugin_message::Message::FieldResponse(proto::FormattedFieldResponse {
                            field: None,
                        })
                    }
                } else {
                    plugin_message::Message::FieldResponse(proto::FormattedFieldResponse {
                        field: None,
                    })
                }
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

impl CodeSnippetExtractorPlugin {
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

impl Default for CodeSnippetExtractorPlugin {
    fn default() -> Self {
        Self::new()
    }
}

lla_plugin_interface::declare_plugin!(CodeSnippetExtractorPlugin);
