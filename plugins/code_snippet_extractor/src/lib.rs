use colored::Colorize;
use lla_plugin_interface::{CliArg, DecoratedEntry, EntryDecorator, Plugin};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

#[derive(Clone)]
pub struct CodeSnippetExtractorPlugin {
    snippet_file: PathBuf,
    snippets: HashMap<String, Vec<(String, String)>>,
}

impl CodeSnippetExtractorPlugin {
    pub fn new() -> Self {
        let snippet_file = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("lla")
            .join("code_snippets.txt");
        let snippets = Self::load_snippets(&snippet_file);
        CodeSnippetExtractorPlugin {
            snippet_file,
            snippets,
        }
    }

    fn load_snippets(path: &PathBuf) -> HashMap<String, Vec<(String, String)>> {
        let mut snippets = HashMap::new();
        if let Ok(file) = File::open(path) {
            let reader = BufReader::new(file);
            let mut current_file = String::new();
            let mut current_snippet = Vec::new();
            let mut current_name = String::new();

            for line in reader.lines().filter_map(|l| l.ok()) {
                if line.starts_with("FILE:") {
                    if !current_file.is_empty() && !current_snippet.is_empty() {
                        snippets
                            .entry(current_file.clone())
                            .or_insert_with(Vec::new)
                            .push((current_name.clone(), current_snippet.join("\n")));
                        current_snippet.clear();
                    }
                    current_file = line[5..].trim().to_string();
                } else if line.starts_with("NAME:") {
                    if !current_name.is_empty() && !current_snippet.is_empty() {
                        snippets
                            .entry(current_file.clone())
                            .or_insert_with(Vec::new)
                            .push((current_name.clone(), current_snippet.join("\n")));
                        current_snippet.clear();
                    }
                    current_name = line[5..].trim().to_string();
                } else {
                    current_snippet.push(line);
                }
            }

            if !current_file.is_empty() && !current_snippet.is_empty() {
                snippets
                    .entry(current_file)
                    .or_insert_with(Vec::new)
                    .push((current_name, current_snippet.join("\n")));
            }
        }
        snippets
    }

    fn save_snippets(&self) {
        if let Some(parent) = self.snippet_file.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        if let Ok(mut file) = File::create(&self.snippet_file) {
            for (file_path, snippets) in &self.snippets {
                for (name, content) in snippets {
                    writeln!(file, "FILE: {}", file_path).ok();
                    writeln!(file, "NAME: {}", name).ok();
                    writeln!(file, "{}", content).ok();
                    writeln!(file).ok();
                }
            }
        }
    }

    fn extract_snippet(
        &mut self,
        file_path: &str,
        name: &str,
        start_line: usize,
        end_line: usize,
    ) -> Result<(), String> {
        let file = File::open(file_path).map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader.lines().filter_map(|l| l.ok()).collect();

        if start_line > end_line || end_line > lines.len() {
            return Err("Invalid line range".to_string());
        }

        let snippet = lines[start_line - 1..end_line].join("\n");
        self.snippets
            .entry(file_path.to_string())
            .or_insert_with(Vec::new)
            .push((name.to_string(), snippet));
        self.save_snippets();
        Ok(())
    }

    fn list_snippets(&self, file_path: &str) -> Vec<String> {
        self.snippets
            .get(file_path)
            .map(|snippets| snippets.iter().map(|(name, _)| name.clone()).collect())
            .unwrap_or_else(Vec::new)
    }

    fn get_snippet(&self, file_path: &str, name: &str) -> Option<String> {
        self.snippets.get(file_path).and_then(|snippets| {
            snippets
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, content)| content.clone())
        })
    }
}

impl Plugin for CodeSnippetExtractorPlugin {
    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn description(&self) -> &'static str {
        env!("CARGO_PKG_DESCRIPTION")
    }

    fn perform_action(&self, action: &str, args: &[String]) -> Result<(), String> {
        match action {
            "extract" => {
                if args.len() != 4 {
                    return Err(
                        "Usage: extract <file_path> <snippet_name> <start_line> <end_line>"
                            .to_string(),
                    );
                }
                let start_line = args[2]
                    .parse()
                    .map_err(|_| "Invalid start line".to_string())?;
                let end_line = args[3]
                    .parse()
                    .map_err(|_| "Invalid end line".to_string())?;
                let mut plugin = self.clone();
                plugin.extract_snippet(&args[0], &args[1], start_line, end_line)
            }
            "list" => {
                if args.len() != 1 {
                    return Err("Usage: list <file_path>".to_string());
                }
                let snippets = self.list_snippets(&args[0]);
                println!("Snippets for {}: {:?}", args[0], snippets);
                Ok(())
            }
            "get" => {
                if args.len() != 2 {
                    return Err("Usage: get <file_path> <snippet_name>".to_string());
                }
                if let Some(snippet) = self.get_snippet(&args[0], &args[1]) {
                    println!("Snippet '{}' from {}:\n{}", args[1], args[0], snippet);
                    Ok(())
                } else {
                    Err(format!("Snippet '{}' not found in {}", args[1], args[0]))
                }
            }
            "help" => {
                println!("Available actions:");
                println!("  extract <file_path> <snippet_name> <start_line> <end_line>");
                println!("  list <file_path>");
                println!("  get <file_path> <snippet_name>");
                Ok(())
            }
            _ => Err(format!("Unknown action: {}", action)),
        }
    }
}
impl EntryDecorator for CodeSnippetExtractorPlugin {
    fn name(&self) -> &'static str {
        env!("CARGO_PKG_NAME")
    }

    fn decorate(&self, entry: &mut DecoratedEntry) {
        if let Some(file_path) = entry.path.to_str() {
            let snippet_count = self.snippets.get(file_path).map(|s| s.len()).unwrap_or(0);
            if snippet_count > 0 {
                entry
                    .custom_fields
                    .insert("snippet_count".to_string(), snippet_count.to_string());
            }
        }
    }

    fn format_field(&self, entry: &DecoratedEntry, _format: &str) -> Option<String> {
        entry
            .custom_fields
            .get("snippet_count")
            .map(|count| format!("[{} snippets]", count.yellow()))
    }
}

lla_plugin_interface::declare_plugin!(CodeSnippetExtractorPlugin);
