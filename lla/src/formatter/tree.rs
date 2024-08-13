use super::FileFormatter;
use crate::error::Result;
use crate::plugin::PluginManager;
use crate::utils::color::colorize_file_name;
use lla_plugin_interface::DecoratedEntry;
use std::fs;

pub struct TreeFormatter;

impl FileFormatter for TreeFormatter {
    fn format_files(
        &self,
        files: &[DecoratedEntry],
        plugin_manager: &PluginManager,
        depth: Option<usize>,
    ) -> Result<String> {
        fn build_tree(
            entry: &DecoratedEntry,
            prefix: &str,
            is_last: bool,
            current_depth: usize,
            max_depth: Option<usize>,
            plugin_manager: &PluginManager,
        ) -> std::io::Result<String> {
            let mut result = String::new();
            let file_name = colorize_file_name(&entry.path);
            let branch = if is_last { "└── " } else { "├── " };

            let plugin_fields = plugin_manager.format_fields(entry, "tree").join(" ");

            result.push_str(&format!(
                "{}{}{} {}\n",
                prefix, branch, file_name, plugin_fields
            ));

            if entry.path.is_dir() && max_depth.map_or(true, |d| current_depth < d) {
                let new_prefix = format!("{}{}   ", prefix, if is_last { " " } else { "│" });
                let entries = fs::read_dir(&entry.path)?;
                let mut entries: Vec<_> = entries
                    .filter_map(|e| e.ok())
                    .map(|e| DecoratedEntry {
                        path: e.path(),
                        metadata: e.metadata().unwrap(),
                        custom_fields: std::collections::HashMap::new(),
                    })
                    .collect();
                entries.sort_by_key(|e| e.path.file_name().unwrap().to_owned());
                let last_index = entries.len().saturating_sub(1);
                for (index, child_entry) in entries.into_iter().enumerate() {
                    result.push_str(&build_tree(
                        &child_entry,
                        &new_prefix,
                        index == last_index,
                        current_depth + 1,
                        max_depth,
                        plugin_manager,
                    )?);
                }
            }
            Ok(result)
        }

        let mut output = String::new();
        for (index, file) in files.iter().enumerate() {
            output.push_str(&build_tree(
                file,
                "",
                index == files.len() - 1,
                0,
                depth,
                plugin_manager,
            )?);
        }
        Ok(output)
    }
}
