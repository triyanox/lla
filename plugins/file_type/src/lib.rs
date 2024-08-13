use lla_plugin_interface::{DecoratedEntry, EntryDecorator, Plugin};
use std::collections::HashMap;
use std::os::unix::fs::PermissionsExt;

pub struct FileTypePlugin {
    icon_map: HashMap<&'static str, &'static str>,
}

impl FileTypePlugin {
    pub fn new() -> Self {
        let mut icon_map = HashMap::new();

        icon_map.insert("rs", "🦀");
        icon_map.insert("js", "🟨");
        icon_map.insert("ts", "🟦");
        icon_map.insert("py", "🐍");
        icon_map.insert("rb", "💎");
        icon_map.insert("php", "🐘");
        icon_map.insert("java", "☕");
        icon_map.insert("cpp", "🚀");
        icon_map.insert("c", "🇨");
        icon_map.insert("go", "🐹");
        icon_map.insert("html", "🌐");
        icon_map.insert("css", "🎨");
        icon_map.insert("json", "📊");
        icon_map.insert("xml", "📑");
        icon_map.insert("yml", "🔧");
        icon_map.insert("yaml", "🔧");
        icon_map.insert("toml", "🔧");
        icon_map.insert("ini", "🔧");
        icon_map.insert("cfg", "🔧");
        icon_map.insert("txt", "📝");
        icon_map.insert("md", "📜");
        icon_map.insert("rst", "📜");
        icon_map.insert("doc", "📘");
        icon_map.insert("docx", "📘");
        icon_map.insert("pdf", "📕");
        icon_map.insert("xls", "📊");
        icon_map.insert("xlsx", "📊");
        icon_map.insert("ppt", "📊");
        icon_map.insert("pptx", "📊");
        icon_map.insert("jpg", "🖼️");
        icon_map.insert("jpeg", "🖼️");
        icon_map.insert("png", "🖼️");
        icon_map.insert("gif", "🖼️");
        icon_map.insert("bmp", "🖼️");
        icon_map.insert("svg", "🖼️");
        icon_map.insert("mp3", "🎵");
        icon_map.insert("wav", "🎵");
        icon_map.insert("flac", "🎵");
        icon_map.insert("mp4", "🎥");
        icon_map.insert("avi", "🎥");
        icon_map.insert("mov", "🎥");
        icon_map.insert("mkv", "🎥");
        icon_map.insert("zip", "📦");
        icon_map.insert("tar", "📦");
        icon_map.insert("gz", "📦");
        icon_map.insert("7z", "📦");
        icon_map.insert("rar", "📦");

        FileTypePlugin { icon_map }
    }
}

impl Plugin for FileTypePlugin {
    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn description(&self) -> &'static str {
        env!("CARGO_PKG_DESCRIPTION")
    }
}

impl EntryDecorator for FileTypePlugin {
    fn name(&self) -> &'static str {
        env!("CARGO_PKG_NAME")
    }

    fn decorate(&self, entry: &mut DecoratedEntry) {
        let icon = if entry.path.is_dir() {
            "📁"
        } else {
            entry
                .path
                .extension()
                .and_then(|ext| ext.to_str())
                .and_then(|ext| self.icon_map.get(ext))
                .unwrap_or_else(|| {
                    if entry.path.is_symlink() {
                        &&"🔗"
                    } else if entry.metadata.permissions().mode() & 0o111 != 0 {
                        &&"🚀"
                    } else {
                        &&"📄"
                    }
                })
        };

        entry
            .custom_fields
            .insert("file_type".to_string(), icon.to_string());
    }

    fn format_field(&self, entry: &DecoratedEntry, _format: &str) -> Option<String> {
        entry.custom_fields.get("file_type").cloned()
    }
}

lla_plugin_interface::declare_plugin!(FileTypePlugin);
