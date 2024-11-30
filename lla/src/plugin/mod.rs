use crate::config::Config;
use crate::error::{LlaError, Result};
use dashmap::DashMap;
use libloading::{Library, Symbol};
use lla_plugin_interface::{CliArg, DecoratedEntry, Plugin};
use once_cell::sync::Lazy;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

type DecorationCache = DashMap<(u64, String), HashMap<String, String>>;
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

    pub fn handle_plugin_args(&self, args: &[String]) {
        for (name, plugin) in &self.plugins {
            if self.enabled_plugins.contains(name) {
                plugin.handle_cli_args(args);
            }
        }
    }

    pub fn perform_plugin_action(
        &self,
        plugin_name: &str,
        action: &str,
        args: &[String],
    ) -> Result<()> {
        if let Some(plugin) = self.plugins.get(plugin_name) {
            if self.enabled_plugins.contains(plugin_name) {
                plugin
                    .perform_action(action, args)
                    .map_err(LlaError::Plugin)
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

    #[allow(dead_code)]
    pub fn get_cli_args(&self) -> Vec<CliArg> {
        self.plugins
            .iter()
            .filter(|(name, _)| self.enabled_plugins.contains(*name))
            .flat_map(|(_, plugin)| plugin.cli_args())
            .collect()
    }

    pub fn list_plugins(&self) -> Vec<(&str, &str, &str)> {
        self.plugins
            .values()
            .map(|p| (p.name(), p.version(), p.description()))
            .collect()
    }

    pub fn load_plugin<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref().canonicalize()?;
        if self.loaded_paths.contains(&path) {
            return Ok(());
        }

        if !self.enabled_plugins.iter().any(|name| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.contains(name))
                .unwrap_or(false)
        }) {
            return Ok(());
        }

        unsafe {
            let library = Library::new(&path)
                .map_err(|e| LlaError::Plugin(format!("Failed to load plugin library: {}", e)))?;

            let constructor: Symbol<unsafe fn() -> *mut dyn Plugin> =
                library.get(b"_plugin_create").map_err(|e| {
                    LlaError::Plugin(format!("Plugin doesn't have a constructor: {}", e))
                })?;

            let plugin = Box::from_raw(constructor());
            let name = plugin.name().to_string();

            if self.plugins.contains_key(&name) {
                return Err(LlaError::Plugin(format!(
                    "Plugin '{}' already loaded",
                    name
                )));
            }

            self.plugins.insert(name.clone(), plugin);
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

    pub fn decorate_entry(&self, entry: &mut DecoratedEntry, format: &str) {
        if self.enabled_plugins.is_empty() || (format != "default" && format != "long") {
            return;
        }

        #[cfg(unix)]
        let file_id = entry.metadata.ino();
        #[cfg(windows)]
        let file_id = entry.metadata.file_index().unwrap_or(0);

        let cache_key = (file_id, format.to_string());

        if let Some(fields) = DECORATION_CACHE.get(&cache_key) {
            entry
                .custom_fields
                .extend(fields.value().iter().map(|(k, v)| (k.clone(), v.clone())));
            return;
        }

        let enabled_plugins: Vec<_> = self
            .plugins
            .iter()
            .filter(|(name, plugin)| {
                self.enabled_plugins.contains(*name) && plugin.supported_formats().contains(&format)
            })
            .collect();

        if enabled_plugins.is_empty() {
            return;
        }

        let mut new_decorations = HashMap::with_capacity(enabled_plugins.len() * 2);

        let plugin_results: Vec<_> = enabled_plugins
            .into_par_iter()
            .map(|(_name, plugin)| {
                let temp_fields = HashMap::with_capacity(2);
                let mut temp_entry = DecoratedEntry {
                    path: entry.path.clone(),
                    metadata: entry.metadata.clone(),
                    custom_fields: temp_fields,
                };
                plugin.decorate(&mut temp_entry);
                temp_entry.custom_fields
            })
            .collect();

        for fields in plugin_results {
            new_decorations.extend(fields);
        }

        if !new_decorations.is_empty() {
            entry
                .custom_fields
                .extend(new_decorations.iter().map(|(k, v)| (k.clone(), v.clone())));
            DECORATION_CACHE.insert(cache_key, new_decorations);
        }
    }

    #[inline]
    pub fn format_fields(&self, entry: &DecoratedEntry, format: &str) -> Vec<String> {
        if self.enabled_plugins.is_empty() || (format != "default" && format != "long") {
            return Vec::new();
        }

        let mut result = Vec::with_capacity(self.enabled_plugins.len());
        for (name, plugin) in &self.plugins {
            if self.enabled_plugins.contains(name) && plugin.supported_formats().contains(&format) {
                if let Some(field) = plugin.format_field(entry, format) {
                    result.push(field);
                }
            }
        }
        result
    }
}
