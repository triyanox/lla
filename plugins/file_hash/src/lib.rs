use colored::Colorize;
use lla_plugin_interface::{DecoratedEntry, EntryDecorator, Plugin};
use sha1::Sha1;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Read};

pub struct FileHashPlugin;

impl FileHashPlugin {
    pub fn new() -> Self {
        FileHashPlugin
    }

    fn calculate_hashes(path: &std::path::Path) -> Option<(String, String)> {
        let file = File::open(path).ok()?;
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).ok()?;

        let sha1 = format!("{:x}", Sha1::digest(&buffer));
        let sha256 = format!("{:x}", Sha256::digest(&buffer));

        Some((sha1, sha256))
    }
}

impl Plugin for FileHashPlugin {
    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn description(&self) -> &'static str {
        env!("CARGO_PKG_DESCRIPTION")
    }
}

impl EntryDecorator for FileHashPlugin {
    fn name(&self) -> &'static str {
        env!("CARGO_PKG_NAME")
    }

    fn decorate(&self, entry: &mut DecoratedEntry) {
        if entry.path.is_file() {
            if let Some((sha1, sha256)) = Self::calculate_hashes(&entry.path) {
                entry.custom_fields.insert("sha1".to_string(), sha1);
                entry.custom_fields.insert("sha256".to_string(), sha256);
            }
        }
    }

    fn format_field(&self, entry: &DecoratedEntry, format: &str) -> Option<String> {
        match format {
            "long" => {
                let sha1 = entry
                    .custom_fields
                    .get("sha1")
                    .map(|s| s[..8].to_string())
                    .unwrap_or_default();
                let sha256 = entry
                    .custom_fields
                    .get("sha256")
                    .map(|s| s[..8].to_string())
                    .unwrap_or_default();
                Some(format!("{} {}", sha1.green(), sha256.yellow()))
            }
            _ => None,
        }
    }

    fn supported_formats(&self) -> Vec<&'static str> {
        vec!["long"]
    }
}

lla_plugin_interface::declare_plugin!(FileHashPlugin);
