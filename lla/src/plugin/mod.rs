use crate::config::Config;
use crate::error::{LlaError, Result};
use libloading::{Library, Symbol};
use lla_plugin_interface::Plugin;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
    libraries: Vec<Library>,
    loaded_paths: HashSet<PathBuf>,
    enabled_plugins: HashSet<String>,
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

    pub fn list_plugins(&self) -> Vec<(&str, &str, &str)> {
        self.plugins
            .values()
            .map(|p| (p.name(), p.version(), p.description()))
            .collect()
    }

    pub fn load_plugin<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref().canonicalize()?;
        if self.loaded_paths.contains(&path) {
            return Ok(()); // Plugin already loaded, skip
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

            if self.config.enabled_plugins.contains(&name) {
                self.enabled_plugins.insert(name);
            }

            Ok(())
        }
    }

    pub fn discover_plugins<P: AsRef<Path>>(&mut self, plugin_dir: P) -> Result<()> {
        let plugin_dir = plugin_dir.as_ref();
        if !plugin_dir.is_dir() {
            return Err(LlaError::Plugin(format!(
                "Plugin directory does not exist: {:?}",
                plugin_dir
            )));
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

    pub fn decorate_entry(&self, entry: &mut lla_plugin_interface::DecoratedEntry, format: &str) {
        for (name, plugin) in &self.plugins {
            if self.enabled_plugins.contains(name) && plugin.supported_formats().contains(&format) {
                plugin.decorate(entry);
            }
        }
    }

    pub fn format_fields(
        &self,
        entry: &lla_plugin_interface::DecoratedEntry,
        format: &str,
    ) -> Vec<String> {
        self.plugins
            .iter()
            .filter(|(name, _)| self.enabled_plugins.contains(*name))
            .filter(|(_, p)| p.supported_formats().contains(&format))
            .filter_map(|(_, p)| p.format_field(entry, format))
            .collect()
    }
}
