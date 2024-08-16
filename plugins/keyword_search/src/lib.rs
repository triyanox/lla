use colored::Colorize;
use dirs::config_dir;
use lla_plugin_interface::{CliArg, DecoratedEntry, EntryDecorator, Plugin};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct KeywordSearchPlugin {
    keywords: Arc<Mutex<Vec<String>>>,
    config_path: PathBuf,
}

impl KeywordSearchPlugin {
    pub fn new() -> Self {
        let config_path = config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("lla")
            .join("keyword_search.txt");
        let keywords = Arc::new(Mutex::new(Self::load_keywords(&config_path)));

        KeywordSearchPlugin {
            keywords,
            config_path,
        }
    }

    fn load_keywords(path: &PathBuf) -> Vec<String> {
        if let Ok(file) = File::open(path) {
            let reader = BufReader::new(file);
            reader.lines().filter_map(|line| line.ok()).collect()
        } else {
            Vec::new()
        }
    }

    fn save_keywords(&self) {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).ok();
        }
        if let Ok(mut file) = File::create(&self.config_path) {
            let keywords = self.keywords.lock().unwrap();
            for keyword in keywords.iter() {
                writeln!(file, "{}", keyword).ok();
            }
        }
    }

    fn search_file(&self, path: &std::path::Path) -> Option<Vec<(usize, String)>> {
        let file = File::open(path).ok()?;
        let reader = BufReader::new(file);
        let keywords = self.keywords.lock().unwrap();

        reader
            .lines()
            .enumerate()
            .filter_map(|(index, line)| {
                line.ok().and_then(|line| {
                    keywords
                        .iter()
                        .find(|k| line.contains(&***k))
                        .map(|_| (index + 1, line))
                })
            })
            .take(5)
            .collect::<Vec<_>>()
            .into()
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
        if let Some(arg) = args.iter().find(|a| a.starts_with("--keywords=")) {
            let keywords = arg
                .trim_start_matches("--keywords=")
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            *self.keywords.lock().unwrap() = keywords;
            self.save_keywords();
        }
    }

    fn perform_action(&self, action: &str, args: &[String]) -> Result<(), String> {
        match action {
            "set-keywords" => {
                *self.keywords.lock().unwrap() = args.to_vec();
                self.save_keywords();
                Ok(())
            }
            "show-keywords" => {
                println!("Current keywords: {:?}", *self.keywords.lock().unwrap());
                Ok(())
            }
            "help" => {
                println!(
                    "Available actions:\n\
                    - set-keywords: Set the keywords to search for\n\
                    - show-keywords: Show the current keywords\n\
                    - help: Show this help message"
                );
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
        if let Some(matches) = entry.path.is_file().then(|| self.search_file(&entry.path)) {
            if let Some(matches) = matches {
                entry.custom_fields.insert(
                    "keyword_matches".to_string(),
                    matches
                        .into_iter()
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
