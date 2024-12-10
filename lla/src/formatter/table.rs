use super::FileFormatter;
use crate::error::Result;
use crate::plugin::PluginManager;
use crate::utils::color::*;
use colored::*;
use lla_plugin_interface::DecoratedEntry;
use std::cmp;
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use std::time::{Duration, UNIX_EPOCH};
use unicode_width::UnicodeWidthStr;

pub struct TableFormatter;

impl TableFormatter {
    const PADDING: usize = 1;

    fn strip_ansi(s: &str) -> String {
        String::from_utf8(strip_ansi_escapes::strip(s).unwrap_or_default()).unwrap_or_default()
    }

    fn visible_width(s: &str) -> usize {
        Self::strip_ansi(s).width()
    }

    fn calculate_column_widths(files: &[DecoratedEntry]) -> [usize; 4] {
        let mut widths = [
            "Permissions".len(),
            "Size".len(),
            "Modified".len(),
            "Name".len(),
        ];

        for entry in files {
            let perms = Permissions::from_mode(entry.metadata.permissions);
            let perms = colorize_permissions(&perms);
            widths[0] = cmp::max(widths[0], Self::visible_width(&perms));

            let size: ColoredString = colorize_size(entry.metadata.size);
            widths[1] = cmp::max(widths[1], Self::visible_width(&size));

            let modified = UNIX_EPOCH + Duration::from_secs(entry.metadata.modified);
            let date = colorize_date(&modified);
            widths[2] = cmp::max(widths[2], Self::visible_width(&date));

            let name = colorize_file_name(&entry.path);
            widths[3] = cmp::max(widths[3], Self::visible_width(&name));
        }

        widths
    }

    fn create_separator(widths: &[usize]) -> String {
        let mut separator = String::new();
        separator.push('├');
        for (i, &width) in widths.iter().enumerate() {
            separator.push_str(&"─".repeat(width + Self::PADDING * 2));
            if i < widths.len() - 1 {
                separator.push('┼');
            }
        }
        separator.push('┤');
        separator.bright_black().to_string()
    }

    fn create_header(widths: &[usize]) -> String {
        let headers = ["Permissions", "Size", "Modified", "Name"];
        let mut header = String::new();
        header.push('│');

        for (&width, &title) in widths.iter().zip(headers.iter()) {
            header.push(' ');
            header.push_str(
                &format!("{:width$}", title, width = width)
                    .bold()
                    .to_string(),
            );
            header.push(' ');
            header.push('│');
        }
        header.bright_black().to_string()
    }

    fn create_top_border(widths: &[usize]) -> String {
        let mut border = String::new();
        border.push('┌');
        for (i, &width) in widths.iter().enumerate() {
            border.push_str(&"─".repeat(width + Self::PADDING * 2));
            if i < widths.len() - 1 {
                border.push('┬');
            }
        }
        border.push('┐');
        border.bright_black().to_string()
    }

    fn create_bottom_border(widths: &[usize]) -> String {
        let mut border = String::new();
        border.push('└');
        for (i, &width) in widths.iter().enumerate() {
            border.push_str(&"─".repeat(width + Self::PADDING * 2));
            if i < widths.len() - 1 {
                border.push('┴');
            }
        }
        border.push('┘');
        border.bright_black().to_string()
    }

    fn format_cell(content: &str, width: usize, align_right: bool) -> String {
        let visible_width = Self::visible_width(content);
        let padding = width.saturating_sub(visible_width);

        if align_right {
            format!("{}{}", " ".repeat(padding), content)
        } else {
            format!("{}{}", content, " ".repeat(padding))
        }
    }
}

impl FileFormatter for TableFormatter {
    fn format_files(
        &self,
        files: &[DecoratedEntry],
        plugin_manager: &mut PluginManager,
        _depth: Option<usize>,
    ) -> Result<String> {
        if files.is_empty() {
            return Ok(String::new());
        }

        let widths = Self::calculate_column_widths(files);

        let mut output = String::new();
        output.push_str(&Self::create_top_border(&widths));
        output.push('\n');
        output.push_str(&Self::create_header(&widths));
        output.push('\n');
        output.push_str(&Self::create_separator(&widths));
        output.push('\n');

        for entry in files {
            let perms = Permissions::from_mode(entry.metadata.permissions);
            let perms = colorize_permissions(&perms);
            let size = colorize_size(entry.metadata.size);
            let modified = UNIX_EPOCH + Duration::from_secs(entry.metadata.modified);
            let date = colorize_date(&modified);
            let name = colorize_file_name(&entry.path);

            let plugin_fields = plugin_manager.format_fields(entry, "table").join(" ");
            let plugin_suffix = if plugin_fields.is_empty() {
                String::new()
            } else {
                format!(" {}", plugin_fields)
            };

            let formatted_perms = Self::format_cell(&perms, widths[0], false);
            let formatted_size = Self::format_cell(&size, widths[1], true);
            let formatted_date = Self::format_cell(&date, widths[2], false);
            let formatted_name = Self::format_cell(&name, widths[3], false);

            output.push_str(&format!(
                "│{pad}{}{pad}│{pad}{}{pad}│{pad}{}{pad}│{pad}{}{pad}│{}\n",
                formatted_perms,
                formatted_size,
                formatted_date,
                formatted_name,
                plugin_suffix,
                pad = " ",
            ));
        }

        output.push_str(&Self::create_bottom_border(&widths));
        output.push('\n');

        Ok(output)
    }
}
