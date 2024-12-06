use colored::Colorize;
use lazy_static::lazy_static;
use lla_plugin_interface::{DecoratedEntry, EntryDecorator, Plugin};
use parking_lot::RwLock;
use rayon::prelude::*;
use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::SystemTime;
use walkdir::WalkDir;

type DirStats = (usize, usize, u64);
type CacheEntry = (SystemTime, DirStats);
type DirCache = HashMap<String, CacheEntry>;

lazy_static! {
    static ref CACHE: RwLock<DirCache> = RwLock::new(HashMap::new());
}

pub struct DirectorySummaryPlugin;

impl DirectorySummaryPlugin {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        DirectorySummaryPlugin
    }

    fn analyze_directory(path: &Path) -> Option<(usize, usize, u64)> {
        let path_str = path.to_string_lossy().to_string();
        if let Ok(metadata) = path.metadata() {
            if let Ok(modified_time) = metadata.modified() {
                let cache = CACHE.read();
                if let Some((cached_time, stats)) = cache.get(&path_str) {
                    if *cached_time >= modified_time {
                        return Some(*stats);
                    }
                }
            }
        }

        let file_count = AtomicUsize::new(0);
        let dir_count = AtomicUsize::new(0);
        let total_size = AtomicU64::new(0);

        WalkDir::new(path)
            .into_iter()
            .par_bridge()
            .filter_map(|e| e.ok())
            .for_each(|entry| {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        file_count.fetch_add(1, Ordering::Relaxed);
                        total_size.fetch_add(metadata.len(), Ordering::Relaxed);
                    } else if metadata.is_dir() {
                        dir_count.fetch_add(1, Ordering::Relaxed);
                    }
                }
            });

        let result = (
            file_count.load(Ordering::Relaxed),
            dir_count.load(Ordering::Relaxed),
            total_size.load(Ordering::Relaxed),
        );
        if let Ok(metadata) = path.metadata() {
            if let Ok(modified_time) = metadata.modified() {
                let mut cache = CACHE.write();
                cache.insert(path_str, (modified_time, result));
            }
        }

        Some(result)
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

impl Default for DirectorySummaryPlugin {
    fn default() -> Self {
        Self::new()
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

                let modified = entry
                    .path
                    .metadata()
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| t.elapsed().ok())
                    .map(|e| {
                        let secs = e.as_secs();
                        if secs < 60 {
                            format!("{} secs ago", secs)
                        } else if secs < 3600 {
                            format!("{} mins ago", secs / 60)
                        } else if secs < 86400 {
                            format!("{} hours ago", secs / 3600)
                        } else {
                            format!("{} days ago", secs / 86400)
                        }
                    })
                    .unwrap_or_else(|| "unknown time".to_string());

                Some(format!(
                    "{} files, {} dirs, {} (modified {})",
                    file_count.bright_cyan(),
                    dir_count.bright_green(),
                    total_size.bright_yellow(),
                    modified.bright_magenta()
                ))
            }
            "default" => {
                let file_count = entry.custom_fields.get("file_count")?;
                let total_size = entry.custom_fields.get("total_size")?;
                Some(format!("{} files, {}", file_count, total_size))
            }
            _ => None,
        }
    }

    fn supported_formats(&self) -> Vec<&'static str> {
        vec!["default", "long"]
    }
}

lla_plugin_interface::declare_plugin!(DirectorySummaryPlugin);
