use super::FileFormatter;
use crate::error::Result;
use crate::plugin::PluginManager;
use crate::utils::color::colorize_file_name;
use colored::*;
use lla_plugin_interface::DecoratedEntry;
use terminal_size::{terminal_size, Width};
use unicode_width::UnicodeWidthStr;

pub struct SizeMapFormatter;

impl SizeMapFormatter {
    const PADDING: usize = 2;

    fn strip_ansi(s: &str) -> String {
        String::from_utf8(strip_ansi_escapes::strip(s).unwrap_or_default()).unwrap_or_default()
    }

    fn visible_width(s: &str) -> usize {
        Self::strip_ansi(s).width()
    }

    fn calculate_widths(files: &[DecoratedEntry], _term_width: usize) -> (usize, usize) {
        let max_name_width = files
            .iter()
            .map(|f| Self::visible_width(&colorize_file_name(&f.path)))
            .max()
            .unwrap_or(20)
            .min(40);

        let size_width = files
            .iter()
            .map(|f| Self::visible_width(&format_size(f.metadata.size)))
            .max()
            .unwrap_or(8)
            .max(8);

        (max_name_width, size_width)
    }

    fn create_header(term_width: usize) -> String {
        let title = " Size Distribution ";
        let side_width = (term_width - title.len()) / 2;
        format!(
            "{}{}{}\n",
            "─".repeat(side_width).bright_black(),
            title.bold().bright_white(),
            "─"
                .repeat(term_width - side_width - title.len())
                .bright_black()
        )
    }

    fn create_bar(percentage: f64, width: usize) -> String {
        let bar_width = width.saturating_sub(8);
        let filled_width = ((percentage / 100.0) * bar_width as f64) as usize;

        let bar_char = if percentage > 75.0 {
            "█".red()
        } else if percentage > 50.0 {
            "█".yellow()
        } else if percentage > 25.0 {
            "█".cyan()
        } else {
            "█".bright_blue()
        };

        let partial_char = if percentage > 75.0 {
            "▓".red()
        } else if percentage > 50.0 {
            "▓".yellow()
        } else if percentage > 25.0 {
            "▓".cyan()
        } else {
            "▓".bright_blue()
        };

        let filled = bar_char.to_string().repeat(filled_width.saturating_sub(1));
        let partial = if filled_width > 0 {
            partial_char.to_string()
        } else {
            String::new()
        };
        let empty = "⋅"
            .repeat(bar_width.saturating_sub(filled_width))
            .bright_black();
        format!("{}{}{} {:>5.1}%", filled, partial, empty, percentage)
    }

    fn format_entry(
        name: &str,
        size: &str,
        bar: &str,
        name_width: usize,
        size_width: usize,
    ) -> String {
        format!(
            "  {:<name_width$} {:>size_width$}  {}",
            name,
            size,
            bar,
            name_width = name_width,
            size_width = size_width
        )
    }
}

impl FileFormatter for SizeMapFormatter {
    fn format_files(
        &self,
        files: &[DecoratedEntry],
        plugin_manager: &mut PluginManager,
        _depth: Option<usize>,
    ) -> Result<String> {
        if files.is_empty() {
            return Ok(String::new());
        }

        let term_width = terminal_size()
            .map(|(Width(w), _)| w as usize)
            .unwrap_or(100);

        let (name_width, size_width) = Self::calculate_widths(files, term_width);
        let max_size = files.iter().map(|f| f.metadata.size).max().unwrap_or(1);
        let bar_width = term_width
            .saturating_sub(name_width)
            .saturating_sub(size_width)
            .saturating_sub(Self::PADDING * 4);

        let mut output = String::new();
        output.push_str(&Self::create_header(term_width));
        output.push('\n');

        let mut files: Vec<_> = files.to_vec();
        files.sort_by_key(|f| std::cmp::Reverse(f.metadata.size));

        for file in &files {
            let name = colorize_file_name(&file.path);
            let size_str = format_size(file.metadata.size).bright_cyan();
            let percentage = (file.metadata.size as f64 / max_size as f64 * 100.0) as f64;
            let bar = Self::create_bar(percentage, bar_width);

            let plugin_fields = plugin_manager.format_fields(file, "sizemap").join(" ");
            let plugin_suffix = if plugin_fields.is_empty() {
                String::new()
            } else {
                format!(" {}", plugin_fields)
            };

            let line = Self::format_entry(&name, &size_str, &bar, name_width, size_width);
            output.push_str(&line);
            output.push_str(&plugin_suffix);
            output.push('\n');
        }

        output.push('\n');
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
