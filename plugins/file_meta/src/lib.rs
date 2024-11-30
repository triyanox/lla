use lla_plugin_interface::{DecoratedEntry, EntryDecorator, Plugin};
use std::fs::Metadata;
use std::os::unix::fs::MetadataExt;
use std::time::SystemTime;

pub struct FileMetadataPlugin;

impl FileMetadataPlugin {
    pub fn new() -> Self {
        FileMetadataPlugin
    }

    fn format_timestamp(timestamp: SystemTime) -> String {
        let datetime: chrono::DateTime<chrono::Local> = timestamp.into();
        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    }
}

impl Plugin for FileMetadataPlugin {
    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn description(&self) -> &'static str {
        env!("CARGO_PKG_DESCRIPTION")
    }
}

impl EntryDecorator for FileMetadataPlugin {
    fn name(&self) -> &'static str {
        env!("CARGO_PKG_NAME")
    }

    fn decorate(&self, entry: &mut DecoratedEntry) {
        let metadata: &Metadata = &entry.metadata;

        entry.custom_fields.insert(
            "accessed".to_string(),
            Self::format_timestamp(metadata.accessed().unwrap_or(SystemTime::UNIX_EPOCH)),
        );
        entry.custom_fields.insert(
            "modified".to_string(),
            Self::format_timestamp(metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH)),
        );
        entry.custom_fields.insert(
            "created".to_string(),
            Self::format_timestamp(metadata.created().unwrap_or(SystemTime::UNIX_EPOCH)),
        );
        entry
            .custom_fields
            .insert("inode".to_string(), metadata.ino().to_string());
        entry
            .custom_fields
            .insert("device".to_string(), metadata.dev().to_string());
        entry
            .custom_fields
            .insert("nlink".to_string(), metadata.nlink().to_string());
        entry
            .custom_fields
            .insert("uid".to_string(), metadata.uid().to_string());
        entry
            .custom_fields
            .insert("gid".to_string(), metadata.gid().to_string());
        entry
            .custom_fields
            .insert("size".to_string(), metadata.size().to_string());
        entry
            .custom_fields
            .insert("blocks".to_string(), metadata.blocks().to_string());
        entry
            .custom_fields
            .insert("blksize".to_string(), metadata.blksize().to_string());
    }

    fn format_field(&self, entry: &DecoratedEntry, format: &str) -> Option<String> {
        match format {
            "long" | "default" => Some(format!(
                "\nAccessed: {}\nModified: {}\nCreated: {}\nInode: {}\nDevice: {}\nLinks: {}\nUID: {}\nGID: {}\nSize: {}\nBlocks: {}\nBlock Size: {}",
                entry.custom_fields.get("accessed").unwrap(),
                entry.custom_fields.get("modified").unwrap(),
                entry.custom_fields.get("created").unwrap(),
                entry.custom_fields.get("inode").unwrap(),
                entry.custom_fields.get("device").unwrap(),
                entry.custom_fields.get("nlink").unwrap(),
                entry.custom_fields.get("uid").unwrap(),
                entry.custom_fields.get("gid").unwrap(),
                entry.custom_fields.get("size").unwrap(),
                entry.custom_fields.get("blocks").unwrap(),
                entry.custom_fields.get("blksize").unwrap(),
            )),
            _ => None,
        }
    }
}

impl Default for FileMetadataPlugin {
    fn default() -> Self {
        Self::new()
    }
}

lla_plugin_interface::declare_plugin!(FileMetadataPlugin);
