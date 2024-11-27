use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::error::LlaError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub default_sort: String,
    pub default_format: String,
    pub enabled_plugins: Vec<String>,
    pub plugins_dir: PathBuf,
    pub default_depth: Option<usize>,
}

impl Config {
    pub fn new() -> Self {
        Config::default()
    }

    pub fn load(path: &Path) -> io::Result<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        if path.exists() {
            let contents = fs::read_to_string(path)?;
            toml::from_str(&contents).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
        } else {
            let config = Config::default();
            config.ensure_plugins_dir()?;
            config.save(path)?;
            Ok(config)
        }
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        self.ensure_plugins_dir()?;

        let contents = toml::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(path, contents)
    }

    pub fn get_config_path() -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(".config").join("lla").join("config.toml")
    }

    pub fn ensure_plugins_dir(&self) -> io::Result<()> {
        fs::create_dir_all(&self.plugins_dir)
    }

    pub fn enable_plugin(&mut self, plugin_name: &str) -> Result<(), LlaError> {
        self.ensure_plugins_dir().map_err(LlaError::Io)?;
        if !self.enabled_plugins.contains(&plugin_name.to_string()) {
            self.enabled_plugins.push(plugin_name.to_string());
            self.save(&Self::get_config_path())?;
        }
        Ok(())
    }

    pub fn disable_plugin(&mut self, plugin_name: &str) -> Result<(), LlaError> {
        self.enabled_plugins.retain(|name| name != plugin_name);
        self.save(&Self::get_config_path())?;
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let default_plugins_dir = home.join(".config").join("lla").join("plugins");

        Config {
            default_sort: String::from("name"),
            default_format: String::from("default"),
            enabled_plugins: vec![],
            plugins_dir: default_plugins_dir,
            default_depth: Some(3),
        }
    }
}

pub fn initialize_config() -> Result<(), LlaError> {
    let config_path = Config::get_config_path();

    if config_path.exists() {
        println!("Config file already exists at {:?}", config_path);
        println!("Use `lla config` to view or modify the configuration.");
        return Ok(());
    }

    let config = Config::default();
    config.ensure_plugins_dir().map_err(LlaError::Io)?;
    config.save(&config_path).map_err(LlaError::Io)?;

    println!("Config file initialized at {:?}", config_path);
    println!("Plugins directory created at {:?}", config.plugins_dir);
    println!("Default configuration:");
    println!("{:#?}", config);

    Ok(())
}

pub fn view_config() -> Result<(), LlaError> {
    let config_path = Config::get_config_path();
    let config = Config::load(&config_path).map_err(LlaError::Io)?;
    println!("Current configuration at {:?}:", config_path);
    println!("{:#?}", config);
    Ok(())
}

pub fn update_config(key: &str, value: &str) -> Result<(), LlaError> {
    let config_path = Config::get_config_path();
    let mut config = Config::load(&config_path).map_err(LlaError::Io)?;

    match key {
        "plugins_dir" => {
            config.plugins_dir = PathBuf::from(value);
            config.ensure_plugins_dir().map_err(LlaError::Io)?;
        }
        "default_sort" => config.default_sort = value.to_string(),
        "default_format" => config.default_format = value.to_string(),
        "default_depth" => {
            config.default_depth = Some(value.parse().map_err(|_| {
                LlaError::Config(format!("Invalid value for default_depth: {}", value))
            })?);
        }
        _ => return Err(LlaError::Config(format!("Unknown config key: {}", key))),
    }

    config.save(&config_path).map_err(LlaError::Io)?;
    println!("Updated {} to {}", key, value);
    Ok(())
}
