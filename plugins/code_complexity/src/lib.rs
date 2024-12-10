use colored::Colorize;
use dirs::config_dir;
use lla_plugin_interface::{DecoratedEntry, Plugin};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use toml;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LanguageRules {
    extensions: Vec<String>,
    function_patterns: Vec<String>,
    class_patterns: Vec<String>,
    branch_patterns: Vec<String>,
    loop_patterns: Vec<String>,
    comment_patterns: Vec<String>,
    max_line_length: usize,
    max_function_lines: usize,
}

impl Default for LanguageRules {
    fn default() -> Self {
        Self {
            extensions: vec!["rs".to_string()],
            function_patterns: vec!["fn ".to_string()],
            class_patterns: vec![
                "struct ".to_string(),
                "impl ".to_string(),
                "trait ".to_string(),
            ],
            branch_patterns: vec!["if ".to_string(), "match ".to_string(), "else".to_string()],
            loop_patterns: vec!["for ".to_string(), "while ".to_string(), "loop".to_string()],
            comment_patterns: vec!["//".to_string(), "/*".to_string()],
            max_line_length: 100,
            max_function_lines: 50,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ComplexityConfig {
    languages: HashMap<String, LanguageRules>,
    thresholds: ComplexityThresholds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ComplexityThresholds {
    low: f32,
    medium: f32,
    high: f32,
    very_high: f32,
}

impl Default for ComplexityThresholds {
    fn default() -> Self {
        Self {
            low: 10.0,
            medium: 20.0,
            high: 30.0,
            very_high: 40.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ComplexityMetrics {
    lines: usize,
    functions: usize,
    classes: usize,
    branches: usize,
    loops: usize,
    comments: usize,
    long_lines: usize,
    long_functions: Vec<(String, usize)>,
    cyclomatic_complexity: usize,
    cognitive_complexity: usize,
    maintainability_index: f32,
}

impl Default for ComplexityMetrics {
    fn default() -> Self {
        Self {
            lines: 0,
            functions: 0,
            classes: 0,
            branches: 0,
            loops: 0,
            comments: 0,
            long_lines: 0,
            long_functions: Vec::new(),
            cyclomatic_complexity: 0,
            cognitive_complexity: 0,
            maintainability_index: 100.0,
        }
    }
}

pub struct CodeComplexityEstimatorPlugin {
    config: ComplexityConfig,
    config_path: PathBuf,
    stats: HashMap<String, Vec<(PathBuf, ComplexityMetrics)>>,
}

impl CodeComplexityEstimatorPlugin {
    pub fn new() -> Self {
        let config_path = config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("lla")
            .join("code_complexity.toml");

        let config = Self::load_config(&config_path).unwrap_or_else(|| {
            let mut languages = HashMap::new();

            languages.insert(
                "Rust".to_string(),
                LanguageRules {
                    extensions: vec!["rs".to_string()],
                    function_patterns: vec!["fn ".to_string()],
                    class_patterns: vec![
                        "struct ".to_string(),
                        "impl ".to_string(),
                        "trait ".to_string(),
                    ],
                    branch_patterns: vec![
                        "if ".to_string(),
                        "match ".to_string(),
                        "else".to_string(),
                    ],
                    loop_patterns: vec![
                        "for ".to_string(),
                        "while ".to_string(),
                        "loop".to_string(),
                    ],
                    comment_patterns: vec!["//".to_string(), "/*".to_string()],
                    max_line_length: 100,
                    max_function_lines: 50,
                },
            );

            languages.insert(
                "Python".to_string(),
                LanguageRules {
                    extensions: vec!["py".to_string()],
                    function_patterns: vec!["def ".to_string()],
                    class_patterns: vec!["class ".to_string()],
                    branch_patterns: vec![
                        "if ".to_string(),
                        "elif ".to_string(),
                        "else:".to_string(),
                    ],
                    loop_patterns: vec!["for ".to_string(), "while ".to_string()],
                    comment_patterns: vec!["#".to_string()],
                    max_line_length: 88,
                    max_function_lines: 50,
                },
            );

            languages.insert(
                "JavaScript".to_string(),
                LanguageRules {
                    extensions: vec!["js".to_string(), "ts".to_string()],
                    function_patterns: vec!["function ".to_string(), "=> ".to_string()],
                    class_patterns: vec!["class ".to_string()],
                    branch_patterns: vec![
                        "if ".to_string(),
                        "else ".to_string(),
                        "switch ".to_string(),
                    ],
                    loop_patterns: vec![
                        "for ".to_string(),
                        "while ".to_string(),
                        "do ".to_string(),
                    ],
                    comment_patterns: vec!["//".to_string(), "/*".to_string()],
                    max_line_length: 80,
                    max_function_lines: 40,
                },
            );

            ComplexityConfig {
                languages,
                thresholds: ComplexityThresholds::default(),
            }
        });

        CodeComplexityEstimatorPlugin {
            config,
            config_path,
            stats: HashMap::new(),
        }
    }

    fn load_config(path: &PathBuf) -> Option<ComplexityConfig> {
        fs::read_to_string(path)
            .ok()
            .and_then(|content| toml::from_str(&content).ok())
    }

    fn save_config(&self) {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).ok();
        }
        if let Ok(content) = toml::to_string_pretty(&self.config) {
            fs::write(&self.config_path, content).ok();
        }
    }

    fn analyze_file(&self, path: &std::path::Path) -> Option<ComplexityMetrics> {
        let extension = path.extension()?.to_str()?;
        let language = self
            .config
            .languages
            .iter()
            .find(|(_, rules)| rules.extensions.iter().any(|ext| ext == extension))?;
        let rules = language.1;

        let file = File::open(path).ok()?;
        let reader = BufReader::new(file);
        let mut metrics = ComplexityMetrics::default();
        let mut current_function = String::new();
        let mut current_function_lines = 0;
        let mut nesting_level = 0;

        for line in reader.lines().map_while(Result::ok) {
            metrics.lines += 1;
            let trimmed = line.trim();

            if trimmed.is_empty() {
                continue;
            }

            if line.len() > rules.max_line_length {
                metrics.long_lines += 1;
            }

            if rules
                .comment_patterns
                .iter()
                .any(|p| trimmed.starts_with(p))
            {
                metrics.comments += 1;
                continue;
            }

            if rules.function_patterns.iter().any(|p| trimmed.contains(p)) {
                if !current_function.is_empty() && current_function_lines > rules.max_function_lines
                {
                    metrics
                        .long_functions
                        .push((current_function.clone(), current_function_lines));
                }
                current_function = trimmed.to_string();
                current_function_lines = 0;
                metrics.functions += 1;
            }
            current_function_lines += 1;

            if rules.class_patterns.iter().any(|p| trimmed.contains(p)) {
                metrics.classes += 1;
            }

            if rules.branch_patterns.iter().any(|p| trimmed.contains(p)) {
                metrics.branches += 1;
                metrics.cyclomatic_complexity += 1;
                metrics.cognitive_complexity += nesting_level + 1;
            }

            if rules.loop_patterns.iter().any(|p| trimmed.contains(p)) {
                metrics.loops += 1;
                metrics.cyclomatic_complexity += 1;
                metrics.cognitive_complexity += nesting_level + 1;
            }

            if trimmed.ends_with('{') {
                nesting_level += 1;
            } else if trimmed.starts_with('}') {
                nesting_level = nesting_level.saturating_sub(1);
            }
        }

        let volume = (metrics.lines as f32 * (metrics.functions + metrics.classes) as f32).ln();
        let cyclomatic = metrics.cyclomatic_complexity as f32;
        let lines = metrics.lines as f32;
        metrics.maintainability_index =
            (171.0 - 5.2 * volume - 0.23 * cyclomatic - 16.2 * lines.ln()).max(0.0) * 100.0 / 171.0;

        Some(metrics)
    }

    fn get_complexity_color(&self, metrics: &ComplexityMetrics) -> colored::Color {
        let score = metrics.cyclomatic_complexity as f32 * 0.4
            + metrics.cognitive_complexity as f32 * 0.3
            + (100.0 - metrics.maintainability_index) * 0.3;

        if score < self.config.thresholds.low {
            colored::Color::Green
        } else if score < self.config.thresholds.medium {
            colored::Color::Yellow
        } else if score < self.config.thresholds.high {
            colored::Color::Red
        } else {
            colored::Color::BrightRed
        }
    }

    fn format_metrics(&self, metrics: &ComplexityMetrics, detailed: bool) -> String {
        let color = self.get_complexity_color(metrics);
        let mut output = format!(
            "\nComplexity: {} (MI: {:.1})",
            metrics.cyclomatic_complexity.to_string().color(color),
            metrics.maintainability_index
        );

        if detailed {
            output.push_str(&format!("\n  Lines: {}", metrics.lines));
            output.push_str(&format!("\n  Functions: {}", metrics.functions));
            output.push_str(&format!("\n  Classes: {}", metrics.classes));
            output.push_str(&format!("\n  Branches: {}", metrics.branches));
            output.push_str(&format!("\n  Loops: {}", metrics.loops));
            output.push_str(&format!("\n  Comments: {}", metrics.comments));
            output.push_str(&format!("\n  Long lines: {}", metrics.long_lines));

            if !metrics.long_functions.is_empty() {
                output.push_str("\n  Long functions:");
                for (name, lines) in &metrics.long_functions {
                    output.push_str(&format!("\n    {} ({} lines)", name, lines));
                }
            }

            if metrics.maintainability_index < 65.0 {
                output.push_str("\n\nSuggestions:");
                if metrics.long_functions.len() > 2 {
                    output.push_str("\n  - Consider breaking down long functions");
                }
                if metrics.comments < metrics.lines / 10 {
                    output.push_str("\n  - Add more documentation");
                }
                if metrics.cyclomatic_complexity > 10 {
                    output.push_str("\n  - Reduce nested conditionals");
                }
                if metrics.cognitive_complexity > 15 {
                    output.push_str("\n  - Simplify complex logic");
                }
            }
        }

        output
    }

    fn generate_report(&self) -> String {
        let mut output = String::new();
        output.push_str("Code Complexity Report\n\n");

        for (language, files) in &self.stats {
            output.push_str(&format!("{}:\n", language.bright_cyan()));

            let mut total_metrics = ComplexityMetrics::default();
            let mut file_count = 0;

            for (path, metrics) in files {
                output.push_str(&format!(
                    "  {}: {} (MI: {:.1})\n",
                    path.display(),
                    metrics.cyclomatic_complexity,
                    metrics.maintainability_index
                ));

                total_metrics.lines += metrics.lines;
                total_metrics.functions += metrics.functions;
                total_metrics.classes += metrics.classes;
                total_metrics.branches += metrics.branches;
                total_metrics.loops += metrics.loops;
                total_metrics.comments += metrics.comments;
                total_metrics.cyclomatic_complexity += metrics.cyclomatic_complexity;
                total_metrics.cognitive_complexity += metrics.cognitive_complexity;
                file_count += 1;
            }

            if file_count > 0 {
                output.push_str(&format!(
                    "\n  Average metrics:\n    Lines per file: {:.1}\n    Cyclomatic complexity: {:.1}\n    Maintainability index: {:.1}\n\n",
                    total_metrics.lines as f32 / file_count as f32,
                    total_metrics.cyclomatic_complexity as f32 / file_count as f32,
                    total_metrics.maintainability_index / file_count as f32
                ));
            }
        }

        output
    }
}

impl Plugin for CodeComplexityEstimatorPlugin {
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
                let mut entry = match DecoratedEntry::try_from(entry.clone()) {
                    Ok(e) => e,
                    Err(e) => {
                        return self.encode_error(&format!("Failed to convert entry: {}", e));
                    }
                };
                if entry.path.is_file() {
                    if let Some(metrics) = self.analyze_file(&entry.path) {
                        entry.custom_fields.insert(
                            "complexity_metrics".to_string(),
                            toml::to_string(&metrics).unwrap_or_default(),
                        );

                        if let Some(ext) = entry.path.extension().and_then(|e| e.to_str()) {
                            if let Some((lang, _)) = self
                                .config
                                .languages
                                .iter()
                                .find(|(_, rules)| rules.extensions.iter().any(|e| e == ext))
                            {
                                self.stats
                                    .entry(lang.clone())
                                    .or_default()
                                    .push((entry.path.clone(), metrics));
                            }
                        }
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
                let formatted = entry
                    .custom_fields
                    .get("complexity_metrics")
                    .and_then(|toml_str| toml::from_str::<ComplexityMetrics>(toml_str).ok())
                    .map(|metrics| self.format_metrics(&metrics, req.format == "long"));
                plugin_message::Message::FieldResponse(proto::FormattedFieldResponse {
                    field: formatted,
                })
            }
            Some(plugin_message::Message::Action(req)) => {
                let result: Result<(), String> = match req.action.as_str() {
                    "set-thresholds" => {
                        if req.args.len() != 4 {
                            return self.encode_error(
                                "Usage: set-thresholds <low> <medium> <high> <very-high>",
                            );
                        }
                        if let (Ok(low), Ok(medium), Ok(high), Ok(very_high)) = (
                            req.args[0].parse::<f32>(),
                            req.args[1].parse::<f32>(),
                            req.args[2].parse::<f32>(),
                            req.args[3].parse::<f32>(),
                        ) {
                            self.config.thresholds = ComplexityThresholds {
                                low,
                                medium,
                                high,
                                very_high,
                            };
                            self.save_config();
                            println!("Updated complexity thresholds");
                            Ok(())
                        } else {
                            Err("Invalid threshold values".to_string())
                        }
                    }
                    "show-report" => {
                        println!("{}", self.generate_report());
                        Ok(())
                    }
                    "help" => {
                        let help_text = "Available actions:\n\
                            set-thresholds <low> <medium> <high> <very-high> - Set complexity thresholds\n\
                            show-report - Show detailed complexity report\n\
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

impl CodeComplexityEstimatorPlugin {
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

impl Default for CodeComplexityEstimatorPlugin {
    fn default() -> Self {
        Self::new()
    }
}

lla_plugin_interface::declare_plugin!(CodeComplexityEstimatorPlugin);
