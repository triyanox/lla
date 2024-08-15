use colored::Colorize;
use lla_plugin_interface::{DecoratedEntry, EntryDecorator, Plugin};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::sync::{Arc, Mutex};

pub struct DuplicateFileDetectorPlugin {
    file_map: Arc<Mutex<HashMap<(u64, String), Vec<std::path::PathBuf>>>>,
}

impl DuplicateFileDetectorPlugin {
    pub fn new() -> Self {
        DuplicateFileDetectorPlugin {
            file_map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn calculate_partial_hash(path: &std::path::Path, size: u64) -> Option<String> {
        let mut file = File::open(path).ok()?;
        let mut buffer = [0u8; 4096];
        let mut hasher = Sha256::new();
        file.read(&mut buffer).ok()?;
        hasher.update(&buffer);
        if size > 8192 {
            file.seek(SeekFrom::Start(size / 2)).ok()?;
            file.read(&mut buffer).ok()?;
            hasher.update(&buffer);
        }
        if size > 4096 {
            file.seek(SeekFrom::End(-4096)).ok()?;
            file.read(&mut buffer).ok()?;
            hasher.update(&buffer);
        }
        Some(format!("{:x}", hasher.finalize()))
    }
}

impl Plugin for DuplicateFileDetectorPlugin {
    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn description(&self) -> &'static str {
        env!("CARGO_PKG_DESCRIPTION")
    }
}

impl EntryDecorator for DuplicateFileDetectorPlugin {
    fn name(&self) -> &'static str {
        env!("CARGO_PKG_NAME")
    }

    fn decorate(&self, entry: &mut DecoratedEntry) {
        if entry.path.is_file() {
            let size = entry.metadata.len();
            if let Some(hash) = Self::calculate_partial_hash(&entry.path, size) {
                let key = (size, hash);
                let mut file_map = self.file_map.lock().unwrap();
                let entry_list = file_map.entry(key).or_insert_with(Vec::new);
                entry_list.push(entry.path.clone());

                if entry_list.len() > 1 {
                    entry
                        .custom_fields
                        .insert("duplicate_count".to_string(), entry_list.len().to_string());
                    entry.custom_fields.insert(
                        "duplicate_paths".to_string(),
                        entry_list
                            .iter()
                            .filter(|&p| p != &entry.path)
                            .map(|p| p.to_string_lossy().into_owned())
                            .collect::<Vec<_>>()
                            .join(", "),
                    );
                }
            }
        }
    }

    fn format_field(&self, entry: &DecoratedEntry, format: &str) -> Option<String> {
        match format {
            "long" => {
                if let Some(count) = entry.custom_fields.get("duplicate_count") {
                    Some(format!("Potential duplicates: {}", count.bright_red()))
                } else {
                    None
                }
            }
            "default" | "tree" => entry
                .custom_fields
                .get("duplicate_count")
                .map(|count| format!("Duplicates: {}", count)),
            _ => None,
        }
    }

    fn supported_formats(&self) -> Vec<&'static str> {
        vec!["default", "long", "tree"]
    }
}

lla_plugin_interface::declare_plugin!(DuplicateFileDetectorPlugin);
