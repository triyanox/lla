mod args;
mod command_handler;
mod config;
mod error;
mod file_utils;
mod filter;
mod formatter;
mod installer;
mod lister;
mod plugin;
mod plugin_utils;
mod sorter;
mod utils;

use args::{Args, Command};
use command_handler::handle_command;
use config::Config;
use error::Result;
use plugin::PluginManager;

fn main() -> Result<()> {
    let (mut config, config_error) = load_config()?;
    let args = Args::parse(&config);

    if let Some(Command::Clean) = args.command {
        println!("🔄 Starting plugin cleaning...");
        let mut plugin_manager = PluginManager::new(config.clone());
        return plugin_manager.clean_plugins();
    }

    let mut plugin_manager = initialize_plugin_manager(&args, &config)?;
    handle_command(&args, &mut config, &mut plugin_manager, config_error)
}

fn load_config() -> Result<(Config, Option<error::LlaError>)> {
    match Config::load(&Config::get_config_path()) {
        Ok(config) => Ok((config, None)),
        Err(e) => {
            let error = error::LlaError::Config(format!("Failed to load config: {}", e));
            Ok((Config::default(), Some(error)))
        }
    }
}

fn initialize_plugin_manager(args: &Args, config: &Config) -> Result<PluginManager> {
    let mut plugin_manager = PluginManager::new(config.clone());
    plugin_manager.discover_plugins(&args.plugins_dir)?;
    Ok(plugin_manager)
}
