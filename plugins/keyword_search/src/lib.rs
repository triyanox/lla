use colored::Colorize;
use lla_plugin_interface::{CliArg, DecoratedEntry, EntryDecorator, Plugin};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::{Arc, Mutex};

pub struct KeywordSearchPlugin {
    keywords: Arc<Mutex<Vec<String>>>,
}

impl KeywordSearchPlugin {
    pub fn new() -> Self {
        KeywordSearchPlugin {
            keywords: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn search_file(&self, path: &std::path::Path) -> Option<Vec<(usize, String)>> {
        let file = File::open(path).ok()?;
        let reader = BufReader::new(file);
        let keywords = self.keywords.lock().unwrap();
        let mut matches = Vec::new();
        for (index, line) in reader.lines().enumerate() {
            if let Ok(line) = line {
                for keyword in keywords.iter() {
                    if line.contains(keyword) {
                        matches.push((index + 1, line.clone()));
                        break;
                    }
                }
            }
            if matches.len() >= 5 {
                break;
            }
        }
        if matches.is_empty() {
            None
        } else {
            Some(matches)
        }
    }
}

impl Plugin for KeywordSearchPlugin {
    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn description(&self) -> &'static str {
        env!("CARGO_PKG_DESCRIPTION")
    }

    fn cli_args(&self) -> Vec<CliArg> {
        vec![CliArg {
            name: "keywords".to_string(),
            short: Some('k'),
            long: Some("keywords".to_string()),
            help: "Comma-separated list of keywords to search for".to_string(),
            takes_value: true,
        }]
    }

    fn handle_cli_args(&self, args: &[String]) {
        for arg in args {
            if arg.starts_with("--keywords=") {
                let keywords = arg
                    .trim_start_matches("--keywords=")
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect();
                let mut current_keywords = self.keywords.lock().unwrap();
                *current_keywords = keywords;
            }
        }
    }

    fn perform_action(&self, action: &str, args: &[String]) -> Result<(), String> {
        match action {
            "set-keywords" => {
                if args.is_empty() {
                    return Err("No keywords provided".to_string());
                }
                let keywords = args.to_vec();
                let mut current_keywords = self.keywords.lock().unwrap();
                *current_keywords = keywords;
                Ok(())
            }
            "show-keywords" => {
                let current_keywords = self.keywords.lock().unwrap();
                println!("Current keywords: {:?}", *current_keywords);
                Ok(())
            }
            _ => Err(format!("Unknown action: {}", action)),
        }
    }
}

impl EntryDecorator for KeywordSearchPlugin {
    fn name(&self) -> &'static str {
        env!("CARGO_PKG_NAME")
    }

    fn decorate(&self, entry: &mut DecoratedEntry) {
        if entry.path.is_file() {
            if let Some(matches) = self.search_file(&entry.path) {
                entry.custom_fields.insert(
                    "keyword_matches".to_string(),
                    matches
                        .iter()
                        .map(|(line, content)| format!("{}:{}", line, content))
                        .collect::<Vec<_>>()
                        .join("\n"),
                );
            }
        }
    }

    fn format_field(&self, entry: &DecoratedEntry, format: &str) -> Option<String> {
        entry
            .custom_fields
            .get("keyword_matches")
            .map(|matches| match format {
                "long" => format!("Keyword matches:\n{}", matches.bright_yellow()),
                _ => format!("Matches: {}", matches.lines().count()),
            })
    }

    fn supported_formats(&self) -> Vec<&'static str> {
        vec!["default", "long", "tree"]
    }
}

lla_plugin_interface::declare_plugin!(KeywordSearchPlugin);
