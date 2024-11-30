use super::FileFormatter;
use crate::error::Result;
use crate::plugin::PluginManager;
use crate::utils::color::colorize_file_name;
use colored::*;
use lla_plugin_interface::DecoratedEntry;
use std::ffi::OsStr;

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
enum TreePart {
    Edge,
    Line,
    Corner,
}

impl TreePart {
    #[inline]
    const fn as_str(self) -> &'static str {
        match self {
            Self::Edge => "├── ",
            Self::Line => "│   ",
            Self::Corner => "└── ",
        }
    }

    fn colored(self) -> ColoredString {
        match self {
            Self::Edge => self.as_str().bright_black(),
            Self::Line => self.as_str().bright_black(),
            Self::Corner => self.as_str().bright_black(),
        }
    }
}

pub struct TreeFormatter;

impl TreeFormatter {
    fn format_entry(
        entry: &DecoratedEntry,
        plugin_manager: &PluginManager,
        prefix: &str,
        has_plugins: bool,
    ) -> String {
        let mut line = String::with_capacity(prefix.len() + 64);
        line.push_str(prefix);
        line.push_str(&colorize_file_name(&entry.path).to_string());

        if has_plugins {
            let plugin_fields = plugin_manager.format_fields(entry, "tree").join(" ");
            if !plugin_fields.is_empty() {
                line.push(' ');
                line.push_str(&plugin_fields);
            }
        }
        line.push('\n');
        line
    }
}

#[derive(Debug)]
struct TreeTrunk {
    stack: Vec<TreePart>,
    last_depth: Option<(usize, bool)>,
}

impl Default for TreeTrunk {
    fn default() -> Self {
        Self {
            stack: Vec::with_capacity(32),
            last_depth: None,
        }
    }
}

impl TreeTrunk {
    #[inline]
    fn get_prefix(&mut self, depth: usize, is_absolute_last: bool) -> String {
        if let Some((last_depth, _)) = self.last_depth {
            if last_depth < self.stack.len() {
                self.stack[last_depth] = TreePart::Line;
            }
        }

        if depth + 1 > self.stack.len() {
            self.stack.resize(depth + 1, TreePart::Line);
        }

        if depth < self.stack.len() {
            self.stack[depth] = if is_absolute_last {
                TreePart::Corner
            } else {
                TreePart::Edge
            };
        }

        self.last_depth = Some((depth, is_absolute_last));

        let mut colored_prefix = String::new();
        for part in self.stack[1..=depth].iter() {
            colored_prefix.push_str(&part.colored().to_string());
        }
        colored_prefix
    }
}

impl FileFormatter for TreeFormatter {
    fn format_files(
        &self,
        files: &[DecoratedEntry],
        plugin_manager: &PluginManager,
        max_depth: Option<usize>,
    ) -> Result<String> {
        if files.is_empty() {
            return Ok(String::new());
        }

        let has_plugins = !plugin_manager.enabled_plugins.is_empty();
        let mut trunk = TreeTrunk::default();

        let mut entries: Vec<_> = files
            .iter()
            .filter_map(|entry| {
                let depth = entry.path.components().count() - 1;
                if max_depth.map_or(true, |max| depth <= max) {
                    Some((
                        entry,
                        depth,
                        entry.path.clone(),
                        entry
                            .path
                            .file_name()
                            .unwrap_or_else(|| OsStr::new(""))
                            .to_owned(),
                    ))
                } else {
                    None
                }
            })
            .collect();

        entries.sort_unstable_by(|a, b| a.2.cmp(&b.2));
        let avg_line_len = entries
            .first()
            .map(|(e, d, _, _)| {
                let name_len = e.path.file_name().map_or(0, |n| n.len());
                let prefix_len = *d * 4;
                name_len + prefix_len + if has_plugins { 20 } else { 0 }
            })
            .unwrap_or(64);
        let mut result = String::with_capacity(entries.len() * avg_line_len);

        const CHUNK_SIZE: usize = 8192;
        for chunk in entries.chunks(CHUNK_SIZE) {
            let chunk_len = chunk.len();
            for (i, (entry, depth, path, _)) in chunk.iter().enumerate() {
                let is_last = if i + 1 < chunk_len {
                    let (next_entry, next_depth, _, _) = &chunk[i + 1];
                    *depth > *next_depth || !next_entry.path.starts_with(path.parent().unwrap_or(path))
                } else {
                    true
                };

                let prefix = trunk.get_prefix(*depth, is_last);
                result.push_str(&Self::format_entry(entry, plugin_manager, &prefix, has_plugins));
            }
        }

        Ok(result)
    }
}
