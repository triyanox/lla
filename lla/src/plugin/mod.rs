use crate::config::Config;
use crate::error::{LlaError, Result};
use dashmap::DashMap;
use libloading::{Library, Symbol};
use lla_plugin_interface::{DecoratedEntry, EntryMetadata, Plugin, PluginRequest, PluginResponse};
use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

type DecorationCache = DashMap<(String, String), HashMap<String, String>>;
static DECORATION_CACHE: Lazy<DecorationCache> = Lazy::new(DashMap::new);

pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
    libraries: Vec<Library>,
    loaded_paths: HashSet<PathBuf>,
    pub enabled_plugins: HashSet<String>,
    config: Config,
}

impl PluginManager {
    pub fn new(config: Config) -> Self {
        let enabled_plugins = HashSet::from_iter(config.enabled_plugins.clone());
        PluginManager {
            plugins: HashMap::new(),
            libraries: Vec::new(),
            loaded_paths: HashSet::new(),
            enabled_plugins,
            config,
        }
    }

    fn _convert_metadata(metadata: &std::fs::Metadata) -> EntryMetadata {
        EntryMetadata {
            size: metadata.len(),
            modified: metadata
                .modified()
                .map(|t| {
                    t.duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                })
                .unwrap_or(0),
            accessed: metadata
                .accessed()
                .map(|t| {
                    t.duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                })
                .unwrap_or(0),
            created: metadata
                .created()
                .map(|t| {
                    t.duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                })
                .unwrap_or(0),
            is_dir: metadata.is_dir(),
            is_file: metadata.is_file(),
            is_symlink: metadata.is_symlink(),
            permissions: metadata.mode(),
            uid: metadata.uid(),
            gid: metadata.gid(),
        }
    }

    pub fn perform_plugin_action(
        &mut self,
        plugin_name: &str,
        action: &str,
        args: &[String],
    ) -> Result<()> {
        if let Some(plugin) = self.plugins.get_mut(plugin_name) {
            if self.enabled_plugins.contains(plugin_name) {
                match plugin.handle_request(PluginRequest::PerformAction(
                    action.to_string(),
                    args.to_vec(),
                )) {
                    PluginResponse::ActionResult(result) => result.map_err(LlaError::Plugin),
                    _ => Err(LlaError::Plugin("Invalid plugin response".to_string())),
                }
            } else {
                Err(LlaError::Plugin(format!(
                    "Plugin '{}' is not enabled",
                    plugin_name
                )))
            }
        } else {
            Err(LlaError::Plugin(format!(
                "Plugin '{}' not found",
                plugin_name
            )))
        }
    }

    pub fn list_plugins(&mut self) -> Vec<(String, String, String)> {
        let mut result = Vec::new();
        for plugin in self.plugins.values_mut() {
            let name = match plugin.handle_request(PluginRequest::GetName) {
                PluginResponse::Name(name) => name,
                _ => continue,
            };
            let version = match plugin.handle_request(PluginRequest::GetVersion) {
                PluginResponse::Version(version) => version,
                _ => continue,
            };
            let description = match plugin.handle_request(PluginRequest::GetDescription) {
                PluginResponse::Description(description) => description,
                _ => continue,
            };
            result.push((name, version, description));
        }
        result
    }

    pub fn load_plugin<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref().canonicalize()?;
        if self.loaded_paths.contains(&path) {
            return Ok(());
        }

        unsafe {
            let library = Library::new(&path)
                .map_err(|e| LlaError::Plugin(format!("Failed to load plugin library: {}", e)))?;

            let constructor: Symbol<unsafe fn() -> *mut dyn Plugin> =
                library.get(b"_plugin_create").map_err(|e| {
                    LlaError::Plugin(format!("Plugin doesn't have a constructor: {}", e))
                })?;

            let mut plugin = Box::from_raw(constructor());
            let name = match plugin.handle_request(PluginRequest::GetName) {
                PluginResponse::Name(name) => name,
                _ => return Err(LlaError::Plugin("Failed to get plugin name".to_string())),
            };

            if self.plugins.contains_key(&name) {
                return Err(LlaError::Plugin(format!(
                    "Plugin '{}' already loaded",
                    name
                )));
            }

            self.plugins.insert(name, plugin);
            self.libraries.push(library);
            self.loaded_paths.insert(path);

            Ok(())
        }
    }

    pub fn discover_plugins<P: AsRef<Path>>(&mut self, plugin_dir: P) -> Result<()> {
        let plugin_dir = plugin_dir.as_ref();
        if !plugin_dir.is_dir() {
            fs::create_dir_all(plugin_dir).map_err(|e| {
                LlaError::Plugin(format!(
                    "Failed to create plugin directory {:?}: {}",
                    plugin_dir, e
                ))
            })?;
        }

        for entry in fs::read_dir(plugin_dir)? {
            let entry = entry?;
            let path = entry.path();
            if let Some(extension) = path.extension() {
                if extension == "so" || extension == "dll" || extension == "dylib" {
                    match self.load_plugin(&path) {
                        Ok(_) => (),
                        Err(e) => eprintln!("Failed to load plugin {:?}: {}", path, e),
                    }
                }
            }
        }

        Ok(())
    }

    pub fn enable_plugin(&mut self, name: &str) -> Result<()> {
        if self.plugins.contains_key(name) {
            self.enabled_plugins.insert(name.to_string());
            self.config.enable_plugin(name)?;
            Ok(())
        } else {
            Err(LlaError::Plugin(format!("Plugin '{}' not found", name)))
        }
    }

    pub fn disable_plugin(&mut self, name: &str) -> Result<()> {
        if self.plugins.contains_key(name) {
            self.enabled_plugins.remove(name);
            self.config.disable_plugin(name)?;
            Ok(())
        } else {
            Err(LlaError::Plugin(format!("Plugin '{}' not found", name)))
        }
    }

    pub fn decorate_entry(&mut self, entry: &mut DecoratedEntry, format: &str) {
        if self.enabled_plugins.is_empty() || (format != "default" && format != "long") {
            return;
        }
        let path_str = entry.path.to_string_lossy().to_string();
        let cache_key = (path_str, format.to_string());
        if let Some(fields) = DECORATION_CACHE.get(&cache_key) {
            entry
                .custom_fields
                .extend(fields.value().iter().map(|(k, v)| (k.clone(), v.clone())));
            return;
        }
        let supported_names: Vec<_> = {
            let mut names = Vec::new();
            for name in self.enabled_plugins.iter() {
                if let Some(plugin) = self.plugins.get_mut(name) {
                    match plugin.handle_request(PluginRequest::GetSupportedFormats) {
                        PluginResponse::SupportedFormats(formats)
                            if formats.contains(&format.to_string()) =>
                        {
                            names.push(name.clone());
                        }
                        _ => {}
                    }
                }
            }
            names
        };
        if supported_names.is_empty() {
            return;
        }

        let mut new_decorations = HashMap::with_capacity(supported_names.len() * 2);
        for name in supported_names {
            if let Some(plugin) = self.plugins.get_mut(&name) {
                if let PluginResponse::Decorated(decorated) =
                    plugin.handle_request(PluginRequest::Decorate(entry.clone()))
                {
                    new_decorations.extend(decorated.custom_fields);
                }
            }
        }
        if !new_decorations.is_empty() {
            entry
                .custom_fields
                .extend(new_decorations.iter().map(|(k, v)| (k.clone(), v.clone())));
            DECORATION_CACHE.insert(cache_key, new_decorations);
        }
    }

    pub fn format_fields(&mut self, entry: &DecoratedEntry, format: &str) -> Vec<String> {
        if self.enabled_plugins.is_empty() || (format != "default" && format != "long") {
            return Vec::new();
        }

        let mut result = Vec::with_capacity(self.enabled_plugins.len());
        for (name, plugin) in self.plugins.iter_mut() {
            if !self.enabled_plugins.contains(name) {
                continue;
            }

            let supports_format = match plugin.handle_request(PluginRequest::GetSupportedFormats) {
                PluginResponse::SupportedFormats(formats) => formats.contains(&format.to_string()),
                _ => false,
            };

            if supports_format {
                match plugin.handle_request(PluginRequest::FormatField(
                    entry.clone(),
                    format.to_string(),
                )) {
                    PluginResponse::FormattedField(Some(field)) => result.push(field),
                    _ => {}
                }
            }
        }
        result
    }
}
