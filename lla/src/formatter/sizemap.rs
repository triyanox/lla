use super::FileFormatter;
use crate::plugin::PluginManager;
use crate::utils::color::colorize_file_name;
use crate::utils::icons::format_with_icon;
use crate::{error::Result, utils::color::colorize_file_name_with_icon};
use colored::*;
use lla_plugin_interface::proto::DecoratedEntry;
use std::path::Path;
use terminal_size::{terminal_size, Width};
use unicode_width::UnicodeWidthStr;

pub struct SizeMapFormatter {
    pub show_icons: bool,
}

impl SizeMapFormatter {
    pub fn new(show_icons: bool) -> Self {
        Self { show_icons }
    }

    fn strip_ansi(s: &str) -> String {
        String::from_utf8(strip_ansi_escapes::strip(s).unwrap_or_default()).unwrap_or_default()
    }

    fn visible_width(s: &str) -> usize {
        Self::strip_ansi(s).width()
    }

    fn calculate_layout(files: &[DecoratedEntry], term_width: usize) -> (usize, usize, usize) {
        let max_name_width = (term_width as f64 * 0.3) as usize;
        let name_width = files
            .iter()
            .map(|f| {
                let path = Path::new(&f.path);
                let colored_name = colorize_file_name(path).to_string();
                let name_with_icon =
                    colorize_file_name_with_icon(path, format_with_icon(path, colored_name, true));
                Self::visible_width(&name_with_icon)
            })
            .max()
            .unwrap_or(20)
            .clamp(20, max_name_width.min(60));

        let size_width = files
            .iter()
            .map(|f| {
                let size = f.metadata.as_ref().map_or(0, |m| m.size);
                Self::visible_width(&format_size(size))
            })
            .max()
            .unwrap_or(8)
            .max(8);

        let spacing = 6;
        let bar_width = term_width
            .saturating_sub(name_width + size_width + spacing)
            .max(20);

        (name_width, size_width, bar_width)
    }

    fn create_bar(percentage: f64, width: usize) -> String {
        let percent_width = 6;
        let bar_width = width.saturating_sub(percent_width);
        let filled_width = ((percentage / 100.0) * bar_width as f64) as usize;

        let (bar_char, partial_char) = if percentage > 75.0 {
            ("█".red(), "▓".red())
        } else if percentage > 50.0 {
            ("█".yellow(), "▓".yellow())
        } else if percentage > 25.0 {
            ("█".cyan(), "▓".cyan())
        } else {
            ("█".bright_blue(), "▓".bright_blue())
        };

        let filled = if filled_width > 0 {
            bar_char.to_string().repeat(filled_width - 1)
        } else {
            String::new()
        };
        let partial = if filled_width > 0 {
            partial_char.to_string()
        } else {
            String::new()
        };
        let empty = "⋅"
            .repeat(bar_width.saturating_sub(filled_width))
            .bright_black();

        format!("{}{}{} {:>4.1}%", filled, partial, empty, percentage)
    }

    fn truncate_name(name: &str, max_width: usize) -> String {
        if Self::visible_width(name) <= max_width {
            return name.to_string();
        }

        let mut result = String::new();
        let mut current_width = 0;
        let mut in_ansi = false;
        let mut ansi_code = String::new();

        for c in name.chars() {
            if c == '\x1B' {
                in_ansi = true;
                ansi_code.push(c);
                continue;
            }

            if in_ansi {
                ansi_code.push(c);
                if c == 'm' {
                    in_ansi = false;
                    result.push_str(&ansi_code);
                    ansi_code.clear();
                }
                continue;
            }

            let char_width = c.to_string().width();
            if current_width + char_width + 3 <= max_width {
                result.push(c);
                current_width += char_width;
            } else {
                break;
            }
        }

        result.push_str("...");
        if !ansi_code.is_empty() {
            result.push_str("\x1B[0m");
        }

        result
    }

    fn format_entry(
        name: &str,
        size: &str,
        bar: &str,
        name_width: usize,
        size_width: usize,
    ) -> String {
        let truncated_name = Self::truncate_name(name, name_width);
        let visible_name_width = Self::visible_width(&truncated_name);
        let padding = " ".repeat(name_width.saturating_sub(visible_name_width));

        format!(
            "  {}{} {:>size_width$}  {}",
            truncated_name,
            padding,
            size,
            bar,
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

        let (name_width, size_width, bar_width) = Self::calculate_layout(files, term_width);

        let total_size: u64 = files
            .iter()
            .map(|f| f.metadata.as_ref().map_or(0, |m| m.size))
            .sum();

        let mut output = String::new();
        output.push_str(&format!(
            "\n{}\n{}\n\n",
            "Size Map".bright_blue().bold(),
            "─".repeat(term_width.min(80)).bright_black()
        ));

        let mut files = files.to_vec();
        files.sort_by_key(|f| std::cmp::Reverse(f.metadata.as_ref().map_or(0, |m| m.size)));

        for file in &files {
            let path = Path::new(&file.path);
            let colored_name = colorize_file_name(path).to_string();
            let name = format_with_icon(path, colored_name, self.show_icons);
            let size = file.metadata.as_ref().map_or(0, |m| m.size);
            let size_str = format_size(size);
            let percentage = if total_size > 0 {
                (size as f64 / total_size as f64) * 100.0
            } else {
                0.0
            };

            let bar = Self::create_bar(percentage, bar_width);

            let plugin_fields = plugin_manager.format_fields(file, "sizemap").join(" ");
            let plugin_suffix = if plugin_fields.is_empty() {
                String::new()
            } else {
                format!(" {}", plugin_fields)
            };

            output.push_str(&Self::format_entry(
                &name,
                &size_str,
                &format!("{}{}", bar, plugin_suffix),
                name_width,
                size_width,
            ));
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
