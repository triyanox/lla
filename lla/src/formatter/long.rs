use super::FileFormatter;
use crate::error::Result;
use crate::plugin::PluginManager;
use crate::utils::color::*;
use lla_plugin_interface::DecoratedEntry;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::os::unix::fs::MetadataExt;
use std::sync::Mutex;
use users::{get_group_by_gid, get_user_by_uid};

static USER_CACHE: Lazy<Mutex<HashMap<u32, String>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static GROUP_CACHE: Lazy<Mutex<HashMap<u32, String>>> = Lazy::new(|| Mutex::new(HashMap::new()));

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

            let user = {
                let mut cache = USER_CACHE.lock().unwrap();
                if let Some(cached_user) = cache.get(&uid) {
                    cached_user.clone()
                } else {
                    let user_str = get_user_by_uid(uid)
                        .map(|u| u.name().to_string_lossy().into_owned())
                        .unwrap_or_else(|| uid.to_string());
                    cache.insert(uid, user_str.clone());
                    user_str
                }
            };

            let group = {
                let mut cache = GROUP_CACHE.lock().unwrap();
                if let Some(cached_group) = cache.get(&gid) {
                    cached_group.clone()
                } else {
                    let group_str = get_group_by_gid(gid)
                        .map(|g| g.name().to_string_lossy().into_owned())
                        .unwrap_or_else(|| gid.to_string());
                    cache.insert(gid, group_str.clone());
                    group_str
                }
            };

            let plugin_fields = plugin_manager.format_fields(entry, "long").join(" ");

            output.push_str(&format!(
                "{} {:>8} {} {:<8} {:<4} {:<4} {}\n",
                permissions,
                size,
                modified,
                colorize_user(&user),
                colorize_group(&group),
                name,
                plugin_fields
            ));
        }
        Ok(output)
    }
}
