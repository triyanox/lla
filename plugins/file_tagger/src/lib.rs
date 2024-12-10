use colored::Colorize;
use lla_plugin_interface::{Plugin, PluginRequest, PluginResponse};
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
                PluginResponse::SupportedFormats(vec!["default".to_string()])
            }
            PluginRequest::Decorate(mut entry) => {
                let tags = self.get_tags(entry.path.to_str().unwrap_or(""));
                if !tags.is_empty() {
                    entry
                        .custom_fields
                        .insert("tags".to_string(), tags.join(", "));
                }
                PluginResponse::Decorated(entry)
            }
            PluginRequest::FormatField(entry, _) => {
                let formatted = entry.custom_fields.get("tags").map(|tags| {
                    format!(
                        "[{}]",
                        tags.split(", ")
                            .map(|t| t.cyan().to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                });
                PluginResponse::FormattedField(formatted)
            }
            PluginRequest::PerformAction(action, args) => match action.as_str() {
                "add-tag" => {
                    if args.len() != 2 {
                        println!("{} add-tag <file_path> <tag>", "Usage:".bright_cyan());
                        return PluginResponse::ActionResult(Ok(()));
                    }
                    self.add_tag(&args[0], &args[1]);
                    println!(
                        "{} tag '{}' to {}",
                        "Added".bright_green(),
                        args[1].cyan(),
                        args[0].bright_blue()
                    );
                    PluginResponse::ActionResult(Ok(()))
                }
                "remove-tag" => {
                    if args.len() != 2 {
                        println!("{} remove-tag <file_path> <tag>", "Usage:".bright_cyan());
                        return PluginResponse::ActionResult(Ok(()));
                    }
                    self.remove_tag(&args[0], &args[1]);
                    println!(
                        "{} tag '{}' from {}",
                        "Removed".bright_green(),
                        args[1].cyan(),
                        args[0].bright_blue()
                    );
                    PluginResponse::ActionResult(Ok(()))
                }
                "list-tags" => {
                    if args.len() != 1 {
                        println!("{} list-tags <file_path>", "Usage:".bright_cyan());
                        return PluginResponse::ActionResult(Ok(()));
                    }
                    let tags = self.get_tags(&args[0]);
                    if tags.is_empty() {
                        println!(
                            "{} No tags found for {}",
                            "Info:".bright_blue(),
                            args[0].bright_yellow()
                        );
                    } else {
                        println!("{} for {}:", "Tags".bright_green(), args[0].bright_blue());
                        for tag in tags {
                            println!("  {} {}", "→".bright_cyan(), tag.bright_yellow());
                        }
                    }
                    PluginResponse::ActionResult(Ok(()))
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
                    PluginResponse::ActionResult(Ok(()))
                }
                _ => {
                    println!("{} Unknown action: {}", "Error:".bright_red(), action);
                    PluginResponse::ActionResult(Ok(()))
                }
            },
        }
    }
}

lla_plugin_interface::declare_plugin!(FileTaggerPlugin);
