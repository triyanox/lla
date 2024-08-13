use colored::Colorize;
use lla_plugin_interface::{DecoratedEntry, EntryDecorator, Plugin};
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct KeywordSearcherPlugin {
    keywords: Vec<String>,
}

impl KeywordSearcherPlugin {
    pub fn new() -> Self {
        let keywords = vec![
            "TODO".to_string(),
            "FIXME".to_string(),
            "HACK".to_string(),
            "BUG".to_string(),
            "DEPRECATED".to_string(),
        ];
        KeywordSearcherPlugin { keywords }
    }

    fn search_keywords(&self, path: &std::path::Path) -> Option<Vec<(String, usize)>> {
        let file = File::open(path).ok()?;
        let reader = BufReader::new(file);
        let mut matches = Vec::new();

        for (line_number, line) in reader.lines().enumerate() {
            if let Ok(content) = line {
                for keyword in &self.keywords {
                    if content.contains(keyword) {
                        matches.push((keyword.clone(), line_number + 1));
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
}

impl Plugin for KeywordSearcherPlugin {
    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn description(&self) -> &'static str {
        env!("CARGO_PKG_DESCRIPTION")
    }
}

impl EntryDecorator for KeywordSearcherPlugin {
    fn name(&self) -> &'static str {
        env!("CARGO_PKG_NAME")
    }

    fn decorate(&self, entry: &mut DecoratedEntry) {
        if entry.path.is_file() {
            if let Some(ext) = entry.path.extension() {
                if [
                    "txt", "md", "rs", "py", "js", "java", "c", "cpp", "go", "html", "css", "xml",
                    "json", "yml", "toml", "sh", "bat",
                ]
                .contains(&ext.to_str().unwrap_or(""))
                {
                    if let Some(matches) = self.search_keywords(&entry.path) {
                        entry.custom_fields.insert(
                            "keyword_matches".to_string(),
                            serde_json::to_string(&matches).unwrap_or_default(),
                        );
                    }
                }
            }
        }
    }

    fn format_field(&self, entry: &DecoratedEntry, format: &str) -> Option<String> {
        match format {
            "long" | "tree" => {
                let matches: Vec<(String, usize)> =
                    serde_json::from_str(entry.custom_fields.get("keyword_matches")?).ok()?;
                if !matches.is_empty() {
                    let summary = matches
                        .iter()
                        .map(|(keyword, line)| format!("{}:{}", keyword, line))
                        .collect::<Vec<_>>()
                        .join(", ");
                    Some(format!("Keywords: {}", summary).red().to_string())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn supported_formats(&self) -> Vec<&'static str> {
        vec!["long", "tree"]
    }
}

lla_plugin_interface::declare_plugin!(KeywordSearcherPlugin);
