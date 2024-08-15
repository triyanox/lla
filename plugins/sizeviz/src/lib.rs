use colored::Colorize;
use lla_plugin_interface::{DecoratedEntry, EntryDecorator, Plugin};
use std::cmp;
use std::env;

pub struct FileSizeVisualizerPlugin;

impl FileSizeVisualizerPlugin {
    pub fn new() -> Self {
        FileSizeVisualizerPlugin
    }

    fn size_to_bar(size: u64, max_size: u64, max_width: usize) -> String {
        let normalized_size = (size as f64 / max_size as f64 * max_width as f64).ceil() as usize;
        let width = cmp::min(normalized_size, max_width);
        let blocks = "█".repeat(width);
        let spaces = " ".repeat(max_width - width);
        format!("{}{}", blocks, spaces)
    }

    fn size_to_color(size: u64) -> colored::Color {
        match size {
            0..=1024 => colored::Color::Green,
            1025..=1_048_576 => colored::Color::Yellow,
            1_048_577..=1_073_741_824 => colored::Color::Red,
            _ => colored::Color::Magenta,
        }
    }
}

impl Plugin for FileSizeVisualizerPlugin {
    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn description(&self) -> &'static str {
        env!("CARGO_PKG_DESCRIPTION")
    }
}

impl EntryDecorator for FileSizeVisualizerPlugin {
    fn name(&self) -> &'static str {
        env!("CARGO_PKG_NAME")
    }

    fn decorate(&self, entry: &mut DecoratedEntry) {
        if entry.path.is_file() {
            let size = entry.metadata.len();
            entry
                .custom_fields
                .insert("size".to_string(), size.to_string());
        }
    }

    fn format_field(&self, entry: &DecoratedEntry, format: &str) -> Option<String> {
        match format {
            "long" | "tree" => {
                let size = entry.custom_fields.get("size")?.parse::<u64>().ok()?;
                let max_size = 1_073_741_824;
                let bar = Self::size_to_bar(size, max_size, 10);
                let color = Self::size_to_color(size);
                Some(bar.color(color).to_string())
            }
            _ => None,
        }
    }

    fn supported_formats(&self) -> Vec<&'static str> {
        vec!["long", "tree"]
    }
}

lla_plugin_interface::declare_plugin!(FileSizeVisualizerPlugin);
