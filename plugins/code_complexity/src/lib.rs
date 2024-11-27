use colored::Colorize;
use lla_plugin_interface::{DecoratedEntry, EntryDecorator, Plugin};
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CodeComplexityEstimatorPlugin;

impl CodeComplexityEstimatorPlugin {
    pub fn new() -> Self {
        CodeComplexityEstimatorPlugin
    }

    fn estimate_complexity(path: &std::path::Path) -> Option<(usize, usize, usize)> {
        let file = File::open(path).ok()?;
        let reader = BufReader::new(file);
        let mut lines = 0;
        let mut functions = 0;
        let mut branches = 0;

        for content in reader.lines().map_while(Result::ok) {
            lines += 1;
            if content.contains("fn ")
                || content.contains("function ")
                || content.contains("def ")
                || content.contains("class ")
                || content.contains("struct ")
                || content.contains("enum ")
                || content.contains("interface ")
                || content.contains("trait ")
                || content.contains("impl ")
            {
                functions += 1;
            }
            if content.contains("if ")
                || content.contains("else ")
                || content.contains("match ")
                || content.contains("for ")
                || content.contains("while ")
            {
                branches += 1;
            }
        }

        Some((lines, functions, branches))
    }

    fn complexity_score(lines: usize, functions: usize, branches: usize) -> (f32, colored::Color) {
        let score = (lines as f32 * 0.1) + (functions as f32 * 0.5) + (branches as f32 * 0.3);
        let color = match score {
            s if s < 10.0 => colored::Color::Green,
            s if s < 20.0 => colored::Color::Yellow,
            s if s < 30.0 => colored::Color::Red,
            _ => colored::Color::BrightRed,
        };
        (score, color)
    }
}

impl Plugin for CodeComplexityEstimatorPlugin {
    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn description(&self) -> &'static str {
        env!("CARGO_PKG_DESCRIPTION")
    }
}

impl EntryDecorator for CodeComplexityEstimatorPlugin {
    fn name(&self) -> &'static str {
        env!("CARGO_PKG_NAME")
    }

    fn decorate(&self, entry: &mut DecoratedEntry) {
        if entry.path.is_file() {
            if let Some(ext) = entry.path.extension() {
                if ["rs", "py", "js", "java", "c", "cpp", "go", "ts"]
                    .contains(&ext.to_str().unwrap_or(""))
                {
                    if let Some((lines, functions, branches)) =
                        Self::estimate_complexity(&entry.path)
                    {
                        entry
                            .custom_fields
                            .insert("code_lines".to_string(), lines.to_string());
                        entry
                            .custom_fields
                            .insert("code_functions".to_string(), functions.to_string());
                        entry
                            .custom_fields
                            .insert("code_branches".to_string(), branches.to_string());
                    }
                }
            }
        }
    }

    fn format_field(&self, entry: &DecoratedEntry, format: &str) -> Option<String> {
        match format {
            "long" | "tree" => {
                let lines = entry
                    .custom_fields
                    .get("code_lines")?
                    .parse::<usize>()
                    .ok()?;
                let functions = entry
                    .custom_fields
                    .get("code_functions")?
                    .parse::<usize>()
                    .ok()?;
                let branches = entry
                    .custom_fields
                    .get("code_branches")?
                    .parse::<usize>()
                    .ok()?;
                let (score, color) = Self::complexity_score(lines, functions, branches);
                Some(format!("Complexity: {:.1}", score).color(color).to_string())
            }
            _ => None,
        }
    }

    fn supported_formats(&self) -> Vec<&'static str> {
        vec!["long", "tree"]
    }
}

lla_plugin_interface::declare_plugin!(CodeComplexityEstimatorPlugin);

impl Default for CodeComplexityEstimatorPlugin {
    fn default() -> Self {
        Self::new()
    }
}
