use super::FileFormatter;
use crate::error::Result;
use crate::plugin::PluginManager;
use crate::utils::color::*;
use lla_plugin_interface::DecoratedEntry;
use std::os::unix::fs::MetadataExt;
use users::{get_group_by_gid, get_user_by_uid};

pub struct LongFormatter;

impl FileFormatter for LongFormatter {
    fn format_files(
        &self,
        files: &[DecoratedEntry],
        plugin_manager: &PluginManager,
        _depth: Option<usize>,
    ) -> Result<String> {
        let mut output = String::new();
        for entry in files {
            let size = colorize_size(entry.metadata.len());
            let permissions = colorize_permissions(&entry.metadata.permissions());
            let modified = colorize_date(&entry.metadata.modified()?);
            let name = colorize_file_name(&entry.path);

            let uid = entry.metadata.uid();
            let gid = entry.metadata.gid();
            let user = get_user_by_uid(uid)
                .map(|u| u.name().to_string_lossy().into_owned())
                .unwrap_or_else(|| uid.to_string());
            let group = get_group_by_gid(gid)
                .map(|g| g.name().to_string_lossy().into_owned())
                .unwrap_or_else(|| gid.to_string());

            let plugin_fields = plugin_manager.format_fields(entry, "long").join(" ");

            output.push_str(&format!(
                "{} {:>8} {} {:<8} {:<4} {:<4} {}\n",
                permissions,
                size,
                modified,
                colorize_user(user.as_str()),
                colorize_group(group.as_str()),
                name,
                plugin_fields
            ));
        }
        Ok(output)
    }
}
