use super::FileFormatter;
use crate::error::Result;
use crate::plugin::PluginManager;
use crate::utils::color::colorize_file_name;
use colored::*;
use lla_plugin_interface::DecoratedEntry;
use crate::config::Config;

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
        self.as_str().bright_black()
    }
}

pub struct TreeFormatter;

impl TreeFormatter {
    fn format_entry(entry: &DecoratedEntry, prefix: &str, buf: &mut String) {
        buf.clear();
        buf.reserve(prefix.len() + entry.path.as_os_str().len() + 1);
        buf.push_str(prefix);
        buf.push_str(&colorize_file_name(&entry.path).to_string());
        buf.push('\n');
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
    fn get_prefix(&mut self, depth: usize, is_absolute_last: bool, buf: &mut String) {
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

        buf.clear();
        buf.reserve(depth * 4);
        for part in self.stack[1..=depth].iter() {
            buf.push_str(&part.colored());
        }
    }
}

impl FileFormatter for TreeFormatter {
    fn format_files(
        &self,
        files: &[DecoratedEntry],
        _plugin_manager: &PluginManager,
        max_depth: Option<usize>,
    ) -> Result<String> {
        if files.is_empty() {
            return Ok(String::new());
        }

        let mut trunk = TreeTrunk::default();

        let mut entries: Vec<_> = files
            .iter()
            .filter_map(|entry| {
                let depth = entry.path.components().count() - 1;
                if max_depth.map_or(true, |max| depth <= max) {
                    Some((entry, depth, entry.path.clone()))
                } else {
                    None
                }
            })
            .collect();

        entries.sort_unstable_by(|a, b| a.2.cmp(&b.2));

        let config = Config::load(&Config::get_config_path()).unwrap_or_default();
        let max_lines = config.formatters.tree.max_lines.unwrap_or(20_000);

        let total_entries = entries.len();
        let start_index = if max_lines > 0 && total_entries > max_lines {
            eprintln!("Note: Showing only the last {} entries out of {}", max_lines, total_entries);
            total_entries - max_lines
        } else {
            0
        };

        let entries = &entries[start_index..];
        
        let avg_line_len = entries
            .first()
            .map(|(e, d, _)| {
                let name_len = e.path.file_name().map_or(0, |n| n.len());
                let prefix_len = *d * 4;
                name_len + prefix_len + 1
            })
            .unwrap_or(64);
        
        let mut result = String::with_capacity(entries.len() * avg_line_len);
        let mut prefix_buf = String::with_capacity(128);
        let mut entry_buf = String::with_capacity(256);

        const CHUNK_SIZE: usize = 8192;
        for chunk in entries.chunks(CHUNK_SIZE) {
            let chunk_len = chunk.len();
            for (i, (entry, depth, path)) in chunk.iter().enumerate() {
                let is_last = if i + 1 < chunk_len {
                    let (next_entry, next_depth, _) = &chunk[i + 1];
                    *depth > *next_depth || !next_entry.path.starts_with(path.parent().unwrap_or(path))
                } else {
                    true
                };

                trunk.get_prefix(*depth, is_last, &mut prefix_buf);
                Self::format_entry(entry, &prefix_buf, &mut entry_buf);
                result.push_str(&entry_buf);
            }
        }

        Ok(result)
    }
}
