use colored::Colorize;
use lla_plugin_interface::{DecoratedEntry, EntryDecorator, Plugin};
use std::collections::HashMap;
use std::env;

pub struct FileCategoryPlugin {
    categories: HashMap<String, (&'static str, colored::Color)>,
}

impl FileCategoryPlugin {
    pub fn new() -> Self {
        let mut categories = HashMap::new();
        categories.insert(
            "txt|md|doc|docx|pdf".to_string(),
            ("Document", colored::Color::Blue),
        );
        categories.insert(
            "jpg|jpeg|png|gif|bmp|svg".to_string(),
            ("Image", colored::Color::Green),
        );
        categories.insert(
            "mp4|avi|mkv|mov|flv".to_string(),
            ("Video", colored::Color::Magenta),
        );
        categories.insert(
            "zip|tar|gz|rar|7z".to_string(),
            ("Archive", colored::Color::Yellow),
        );
        categories.insert(
            "mp3|wav|ogg|flac".to_string(),
            ("Audio", colored::Color::Cyan),
        );
        categories.insert(
            "exe|msi|dmg|app".to_string(),
            ("Executable", colored::Color::Red),
        );
        categories.insert(
            "html|css|js|json|xml".to_string(),
            ("Web", colored::Color::White),
        );

        FileCategoryPlugin { categories }
    }

    fn get_category(&self, path: &std::path::Path) -> Option<(&'static str, colored::Color)> {
        let extension = path.extension()?.to_str()?.to_lowercase();
        for (extensions, category) in &self.categories {
            if extensions.split('|').any(|ext| ext == extension) {
                return Some(*category);
            }
        }
        None
    }
}

impl Plugin for FileCategoryPlugin {
    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn description(&self) -> &'static str {
        env!("CARGO_PKG_DESCRIPTION")
    }
}

impl EntryDecorator for FileCategoryPlugin {
    fn name(&self) -> &'static str {
        env!("CARGO_PKG_NAME")
    }

    fn decorate(&self, entry: &mut DecoratedEntry) {
        if let Some((category, color)) = self.get_category(&entry.path) {
            entry
                .custom_fields
                .insert("category".to_string(), category.to_string());
            entry
                .custom_fields
                .insert("category_color".to_string(), format!("{:?}", color));
        }
    }

    fn format_field(&self, entry: &DecoratedEntry, format: &str) -> Option<String> {
        match format {
            "default" | "long" | "tree" => {
                let category = entry.custom_fields.get("category")?;
                let color_str = entry.custom_fields.get("category_color")?;
                let color = color_str.parse::<colored::Color>().ok()?;
                Some(format!("[{}]", category.color(color)))
            }
            _ => None,
        }
    }

    fn supported_formats(&self) -> Vec<&'static str> {
        vec!["default", "long", "tree"]
    }
}

lla_plugin_interface::declare_plugin!(FileCategoryPlugin);
