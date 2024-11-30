use super::FileFormatter;
use crate::error::Result;
use crate::plugin::PluginManager;
use crate::utils::color::colorize_file_name;
use colored::*;
use lla_plugin_interface::DecoratedEntry;
use terminal_size::{terminal_size, Width};

pub struct SizeMapFormatter;

impl FileFormatter for SizeMapFormatter {
    fn format_files(
        &self,
        files: &[DecoratedEntry],
        plugin_manager: &PluginManager,
        _depth: Option<usize>,
    ) -> Result<String> {
        if files.is_empty() {
            return Ok(String::new());
        }

        let term_width = terminal_size().map(|(Width(w), _)| w as usize).unwrap_or(80);
        
        let max_name_width = files
            .iter()
            .map(|f| {
                f.path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .len()
            })
            .max()
            .unwrap_or(0)
            .min(40);

        let max_size = files.iter().map(|f| f.metadata.len()).max().unwrap_or(1);
        let size_column_width = 10; 
        let percentage_width = 15; 
        
        let header_text = "Relative Size";
        let bar_width = term_width
            .saturating_sub(max_name_width)
            .saturating_sub(size_column_width)
            .saturating_sub(percentage_width)
            .saturating_sub(5);
        
        let mut output = format!(
            "{:<width$} {:>10} {}\n{}\n",
            "Filename".bold(),
            "Size".bold(),
            header_text.bold(),
            "=".repeat(term_width.min(max_name_width + size_column_width + bar_width + 3)),
            width = max_name_width
        );

        let mut files: Vec<_> = files.to_vec();
        files.sort_by_key(|f| std::cmp::Reverse(f.metadata.len()));

        for file in &files {
            let size = file.metadata.len();
            let name = colorize_file_name(&file.path);
            let plugin_fields = plugin_manager.format_fields(file, "sizemap").join(" ");
            
            let this_bar_width = if max_size == 0 {
                0
            } else {
                ((size as f64 / max_size as f64) * bar_width as f64) as usize
            };

            let percentage = (size as f64 / max_size as f64 * 100.0) as usize;
            let bar = if this_bar_width > 0 {
                format!("{}▏{:>3}%", "█".repeat(this_bar_width).blue(), percentage)
            } else {
                format!("{:>4}%", 0)
            };
            
            let size_str = format_size(size);
            let plugin_suffix = if plugin_fields.is_empty() {
                String::new()
            } else {
                format!(" {}", plugin_fields)
            };
            let name_width = name.chars().count();
            let padding = if name_width < max_name_width {
                " ".repeat(max_name_width - name_width)
            } else {
                String::new()
            };

            let line = format!(
                "{}{} {:>10} {}{}",
                name,
                padding,
                size_str,
                bar,
                plugin_suffix
            );
            
            output.push_str(&line);
            output.push('\n');
        }

        Ok(output)
    }
}

fn format_size(size: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    if size == 0 {
        return "0 B".to_string();
    }

    let size = size as f64;
    let base = 1024_f64;
    let exp = (size.ln() / base.ln()).floor() as i32;
    let exp = exp.clamp(0, (UNITS.len() - 1) as i32);
    
    let size = size / base.powi(exp);
    if exp == 0 {
        format!("{:.0} {}", size, UNITS[exp as usize])
    } else {
        format!("{:.1} {}", size, UNITS[exp as usize])
    }
} 