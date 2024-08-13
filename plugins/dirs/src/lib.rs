use colored::Colorize;
use lla_plugin_interface::{DecoratedEntry, EntryDecorator, Plugin};
use std::env;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub struct DirectorySummaryPlugin;

impl DirectorySummaryPlugin {
    pub fn new() -> Self {
        DirectorySummaryPlugin
    }

    fn analyze_directory(path: &Path) -> Option<(usize, usize, u64)> {
        let mut file_count = 0;
        let mut dir_count = 0;
        let mut total_size = 0;

        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            let metadata = entry.metadata().ok()?;
            if metadata.is_file() {
                file_count += 1;
                total_size += metadata.len();
            } else if metadata.is_dir() {
                dir_count += 1;
            }
        }

        Some((file_count, dir_count, total_size))
    }

    fn format_size(size: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if size >= GB {
            format!("{:.2} GB", size as f64 / GB as f64)
        } else if size >= MB {
            format!("{:.2} MB", size as f64 / MB as f64)
        } else if size >= KB {
            format!("{:.2} KB", size as f64 / KB as f64)
        } else {
            format!("{} B", size)
        }
    }
}

impl Plugin for DirectorySummaryPlugin {
    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn description(&self) -> &'static str {
        env!("CARGO_PKG_DESCRIPTION")
    }
}

impl EntryDecorator for DirectorySummaryPlugin {
    fn name(&self) -> &'static str {
        env!("CARGO_PKG_NAME")
    }

    fn decorate(&self, entry: &mut DecoratedEntry) {
        if entry.path.is_dir() {
            if let Some((file_count, dir_count, total_size)) = Self::analyze_directory(&entry.path)
            {
                entry
                    .custom_fields
                    .insert("file_count".to_string(), file_count.to_string());
                entry
                    .custom_fields
                    .insert("dir_count".to_string(), dir_count.to_string());
                entry
                    .custom_fields
                    .insert("total_size".to_string(), Self::format_size(total_size));
            }
        }
    }

    fn format_field(&self, entry: &DecoratedEntry, format: &str) -> Option<String> {
        if !entry.path.is_dir() {
            return None;
        }

        match format {
            "long" => {
                let file_count = entry.custom_fields.get("file_count")?;
                let dir_count = entry.custom_fields.get("dir_count")?;
                let total_size = entry.custom_fields.get("total_size")?;
                Some(format!(
                    "{} files, {} dirs, {}",
                    file_count.bright_cyan(),
                    dir_count.bright_green(),
                    total_size.bright_yellow()
                ))
            }
            "default" | "tree" => {
                let file_count = entry.custom_fields.get("file_count")?;
                let total_size = entry.custom_fields.get("total_size")?;
                Some(format!("{} files, {}", file_count, total_size))
            }
            _ => None,
        }
    }

    fn supported_formats(&self) -> Vec<&'static str> {
        vec!["default", "long", "tree"]
    }
}

lla_plugin_interface::declare_plugin!(DirectorySummaryPlugin);
