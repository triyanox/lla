use crate::config::Config;
use crate::error::{LlaError, Result};
use dashmap::DashMap;
use libloading::Library;
use lla_plugin_interface::{
    proto::{self, plugin_message::Message, PluginMessage},
    PluginApi, CURRENT_PLUGIN_API_VERSION,
};
use once_cell::sync::Lazy;
use prost::Message as _;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

type DecorationCache = DashMap<(String, String), HashMap<String, String>>;
static DECORATION_CACHE: Lazy<DecorationCache> = Lazy::new(DashMap::new);

pub struct PluginManager {
    plugins: HashMap<String, (Library, *mut PluginApi)>,
    loaded_paths: HashSet<PathBuf>,
    pub enabled_plugins: HashSet<String>,
    config: Config,
}

impl PluginManager {
    pub fn new(config: Config) -> Self {
        let enabled_plugins = HashSet::from_iter(config.enabled_plugins.clone());
        PluginManager {
            plugins: HashMap::new(),
            loaded_paths: HashSet::new(),
            enabled_plugins,
            config,
        }
    }

    fn _convert_metadata(metadata: &std::fs::Metadata) -> proto::EntryMetadata {
        proto::EntryMetadata {
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

    fn send_request(&self, plugin_name: &str, request: PluginMessage) -> Result<PluginMessage> {
        if let Some((_, api)) = self.plugins.get(plugin_name) {
            let mut buf = Vec::with_capacity(request.encoded_len());
            request.encode(&mut buf).unwrap();

            unsafe {
                let raw_response =
                    ((**api).handle_request)(std::ptr::null_mut(), buf.as_ptr(), buf.len());
                let response_vec =
                    Vec::from_raw_parts(raw_response.ptr, raw_response.len, raw_response.capacity);
                let response_msg = proto::PluginMessage::decode(&response_vec[..])
                    .map_err(|e| LlaError::Plugin(format!("Failed to decode response: {}", e)))?;
                Ok(response_msg)
            }
        } else {
            Err(LlaError::Plugin(format!(
                "Plugin '{}' not found",
                plugin_name
            )))
        }
    }

    pub fn perform_plugin_action(
        &mut self,
        plugin_name: &str,
        action: &str,
        args: &[String],
    ) -> Result<()> {
        if !self.enabled_plugins.contains(plugin_name) {
            return Err(LlaError::Plugin(format!(
                "Plugin '{}' is not enabled",
                plugin_name
            )));
        }

        let request = PluginMessage {
            message: Some(Message::Action(proto::ActionRequest {
                action: action.to_string(),
                args: args.to_vec(),
            })),
        };

        match self.send_request(plugin_name, request)?.message {
            Some(Message::ActionResponse(response)) => {
                if response.success {
                    Ok(())
                } else {
                    Err(LlaError::Plugin(
                        response
                            .error
                            .unwrap_or_else(|| "Unknown error".to_string()),
                    ))
                }
            }
            _ => Err(LlaError::Plugin("Invalid response type".to_string())),
        }
    }

    pub fn list_plugins(&mut self) -> Vec<(String, String, String)> {
        let mut result = Vec::new();
        for plugin_name in self.plugins.keys() {
            let name = match self
                .send_request(
                    plugin_name,
                    PluginMessage {
                        message: Some(Message::GetName(true)),
                    },
                )
                .and_then(|msg| match msg.message {
                    Some(Message::NameResponse(name)) => Ok(name),
                    _ => Err(LlaError::Plugin("Invalid response type".to_string())),
                }) {
                Ok(name) => name,
                Err(_) => continue,
            };

            let version = match self
                .send_request(
                    plugin_name,
                    PluginMessage {
                        message: Some(Message::GetVersion(true)),
                    },
                )
                .and_then(|msg| match msg.message {
                    Some(Message::VersionResponse(version)) => Ok(version),
                    _ => Err(LlaError::Plugin("Invalid response type".to_string())),
                }) {
                Ok(version) => version,
                Err(_) => continue,
            };

            let description = match self
                .send_request(
                    plugin_name,
                    PluginMessage {
                        message: Some(Message::GetDescription(true)),
                    },
                )
                .and_then(|msg| match msg.message {
                    Some(Message::DescriptionResponse(description)) => Ok(description),
                    _ => Err(LlaError::Plugin("Invalid response type".to_string())),
                }) {
                Ok(description) => description,
                Err(_) => continue,
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

            let create_fn = library
                .get::<unsafe fn() -> *mut PluginApi>(b"_plugin_create")
                .map_err(|e| {
                    LlaError::Plugin(format!("Plugin doesn't have a create function: {}", e))
                })?;

            let api = create_fn();

            if (*api).version != CURRENT_PLUGIN_API_VERSION {
                return Err(LlaError::Plugin(format!(
                    "Plugin API version mismatch: expected {}, got {}",
                    CURRENT_PLUGIN_API_VERSION,
                    (*api).version
                )));
            }

            let request = PluginMessage {
                message: Some(Message::GetName(true)),
            };
            let mut buf = Vec::with_capacity(request.encoded_len());
            request.encode(&mut buf).unwrap();

            let raw_response =
                ((*api).handle_request)(std::ptr::null_mut(), buf.as_ptr(), buf.len());
            let response_vec =
                Vec::from_raw_parts(raw_response.ptr, raw_response.len, raw_response.capacity);
            let response_msg = proto::PluginMessage::decode(&response_vec[..])
                .map_err(|e| LlaError::Plugin(format!("Failed to decode response: {}", e)))?;

            let name = match response_msg.message {
                Some(Message::NameResponse(name)) => name,
                _ => return Err(LlaError::Plugin("Failed to get plugin name".to_string())),
            };

            if self.plugins.contains_key(&name) {
                return Err(LlaError::Plugin(format!(
                    "Plugin '{}' already loaded",
                    name
                )));
            }

            self.plugins.insert(name, (library, api));
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

    pub fn decorate_entry(&mut self, entry: &mut proto::DecoratedEntry, format: &str) {
        if self.enabled_plugins.is_empty() || (format != "default" && format != "long") {
            return;
        }

        let path_str = entry.path.clone();
        let cache_key = (path_str.clone(), format.to_string());
        if let Some(fields) = DECORATION_CACHE.get(&cache_key) {
            entry
                .custom_fields
                .extend(fields.value().iter().map(|(k, v)| (k.clone(), v.clone())));
            return;
        }

        let supported_names: Vec<_> = {
            let mut names = Vec::new();
            for name in self.enabled_plugins.iter() {
                let request = PluginMessage {
                    message: Some(Message::GetSupportedFormats(true)),
                };

                if let Ok(response) = self.send_request(name, request) {
                    if let Some(Message::FormatsResponse(formats_response)) = response.message {
                        if formats_response.formats.contains(&format.to_string()) {
                            names.push(name.clone());
                        }
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
            let request = PluginMessage {
                message: Some(Message::Decorate(entry.clone())),
            };

            if let Ok(response) = self.send_request(&name, request) {
                if let Some(Message::DecoratedResponse(decorated)) = response.message {
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

    pub fn format_fields(&mut self, entry: &proto::DecoratedEntry, format: &str) -> Vec<String> {
        if self.enabled_plugins.is_empty() || (format != "default" && format != "long") {
            return Vec::new();
        }

        let mut result = Vec::with_capacity(self.enabled_plugins.len());
        for name in self.enabled_plugins.iter() {
            let supports_format = match self.send_request(
                name,
                PluginMessage {
                    message: Some(Message::GetSupportedFormats(true)),
                },
            ) {
                Ok(response) => {
                    if let Some(Message::FormatsResponse(formats)) = response.message {
                        formats.formats.contains(&format.to_string())
                    } else {
                        false
                    }
                }
                Err(_) => false,
            };

            if supports_format {
                let request = PluginMessage {
                    message: Some(Message::FormatField(proto::FormatFieldRequest {
                        entry: Some(entry.clone()),
                        format: format.to_string(),
                    })),
                };

                if let Ok(response) = self.send_request(name, request) {
                    if let Some(Message::FieldResponse(field_response)) = response.message {
                        if let Some(field) = field_response.field {
                            result.push(field);
                        }
                    }
                }
            }
        }
        result
    }
}
