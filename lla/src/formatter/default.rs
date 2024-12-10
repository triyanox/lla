use super::FileFormatter;
use crate::error::Result;
use crate::plugin::PluginManager;
use crate::utils::color::colorize_file_name;
use lla_plugin_interface::DecoratedEntry;

pub struct DefaultFormatter;

impl FileFormatter for DefaultFormatter {
    fn format_files(
        &self,
        files: &[DecoratedEntry],
        plugin_manager: &mut PluginManager,
        _depth: Option<usize>,
    ) -> Result<String> {
        Ok(files
            .iter()
            .map(|file| {
                let colored_name = colorize_file_name(&file.path).to_string();
                let plugin_fields = plugin_manager.format_fields(file, "default").join(" ");
                if plugin_fields.is_empty() {
                    colored_name
                } else {
                    format!("{} {}", colored_name, plugin_fields)
                }
            })
            .collect::<Vec<_>>()
            .join("\n"))
    }
}
