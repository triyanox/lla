use colored::Colorize;
use lla_plugin_interface::{CliArg, DecoratedEntry, EntryDecorator, Plugin};
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
    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn description(&self) -> &'static str {
        env!("CARGO_PKG_DESCRIPTION")
    }

    fn cli_args(&self) -> Vec<CliArg> {
        vec![CliArg {
            name: "tag".to_string(),
            short: Some('g'),
            long: Some("tag".to_string()),
            help: "Filter files by tag".to_string(),
            takes_value: true,
        }]
    }

    fn perform_action(&self, action: &str, args: &[String]) -> Result<(), String> {
        let mut plugin = FileTaggerPlugin::new();
        match action {
            "add-tag" => {
                if args.len() != 2 {
                    return Err("Usage: add-tag <file_path> <tag>".to_string());
                }
                plugin.add_tag(&args[0], &args[1]);
                Ok(())
            }
            "remove-tag" => {
                if args.len() != 2 {
                    return Err("Usage: remove-tag <file_path> <tag>".to_string());
                }
                plugin.remove_tag(&args[0], &args[1]);
                Ok(())
            }
            "list-tags" => {
                if args.len() != 1 {
                    return Err("Usage: list-tags <file_path>".to_string());
                }
                let tags = plugin.get_tags(&args[0]);
                println!("Tags for {}: {:?}", args[0], tags);
                Ok(())
            }
            "help" => Ok(println!(
                "Available actions:\n\
                 - add-tag <file_path> <tag>\n\
                 - remove-tag <file_path> <tag>\n\
                 - list-tags <file_path>"
            )),
            _ => Err(format!("Unknown action: {}", action)),
        }
    }
}

impl EntryDecorator for FileTaggerPlugin {
    fn name(&self) -> &'static str {
        env!("CARGO_PKG_NAME")
    }

    fn decorate(&self, entry: &mut DecoratedEntry) {
        let tags = self.get_tags(entry.path.to_str().unwrap_or(""));
        if !tags.is_empty() {
            entry
                .custom_fields
                .insert("tags".to_string(), tags.join(", "));
        }
    }

    fn format_field(&self, entry: &DecoratedEntry, _format: &str) -> Option<String> {
        entry.custom_fields.get("tags").map(|tags| {
            format!(
                "[{}]",
                tags.split(", ")
                    .map(|t| t.cyan().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })
    }
}

lla_plugin_interface::declare_plugin!(FileTaggerPlugin);
