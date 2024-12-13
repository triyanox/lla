use crate::args::{Args, Command, ConfigAction, InstallSource, ShortcutAction};
use crate::config::{self, Config};
use crate::error::{LlaError, Result};
use crate::file_utils::list_directory;
use crate::installer::PluginInstaller;
use crate::plugin::PluginManager;
use crate::plugin_utils::{handle_plugin_action, list_plugins};
use colored::*;

pub fn handle_command(
    args: &Args,
    config: &mut Config,
    plugin_manager: &mut PluginManager,
    config_error: Option<LlaError>,
) -> Result<()> {
    match &args.command {
        Some(Command::Shortcut(action)) => handle_shortcut_action(action, config),
        Some(Command::Install(source)) => handle_install(source, args),
        Some(Command::Update(plugin_name)) => {
            let installer = PluginInstaller::new(&args.plugins_dir);
            installer.update_plugins(plugin_name.as_deref())
        }
        Some(Command::ListPlugins) => list_plugins(plugin_manager),
        Some(Command::Use) => list_plugins(plugin_manager),
        Some(Command::InitConfig) => config::initialize_config(),
        Some(Command::Config(action)) => handle_config_action(action),
        Some(Command::PluginAction(plugin_name, action, action_args)) => {
            plugin_manager.perform_plugin_action(plugin_name, action, action_args)
        }
        Some(Command::Clean) => unreachable!(),
        None => list_directory(args, plugin_manager, config_error),
    }
}

fn handle_shortcut_action(action: &ShortcutAction, config: &mut Config) -> Result<()> {
    match action {
        ShortcutAction::Add(name, command) => {
            config.add_shortcut(name.clone(), command.clone())?;
            println!(
                "✓ Added shortcut '{}' -> {} {}",
                name.green(),
                command.plugin_name.cyan(),
                command.action.cyan()
            );
            if let Some(desc) = &command.description {
                println!("  Description: {}", desc);
            }
            Ok(())
        }
        ShortcutAction::Remove(name) => {
            if config.get_shortcut(name).is_some() {
                config.remove_shortcut(name)?;
                println!("✓ Removed shortcut '{}'", name.green());
            } else {
                println!("✗ Shortcut '{}' not found", name.red());
            }
            Ok(())
        }
        ShortcutAction::List => {
            if config.shortcuts.is_empty() {
                println!("No shortcuts configured");
                return Ok(());
            }
            println!("\n{}", "Configured Shortcuts:".cyan().bold());
            for (name, cmd) in &config.shortcuts {
                println!(
                    "\n{} → {} {}",
                    name.green(),
                    cmd.plugin_name.cyan(),
                    cmd.action.cyan()
                );
                if let Some(desc) = &cmd.description {
                    println!("  Description: {}", desc);
                }
            }
            println!();
            Ok(())
        }
        ShortcutAction::Run(name, args) => match config.get_shortcut(name) {
            Some(shortcut) => {
                let plugin_name = shortcut.plugin_name.clone();
                let action = shortcut.action.clone();
                handle_plugin_action(config, &plugin_name, &action, args)
            }
            None => {
                println!("✗ Shortcut '{}' not found", name.red());
                Ok(())
            }
        },
    }
}

fn handle_install(source: &InstallSource, args: &Args) -> Result<()> {
    let installer = PluginInstaller::new(&args.plugins_dir);
    match source {
        InstallSource::GitHub(url) => installer.install_from_git(url),
        InstallSource::LocalDir(dir) => installer.install_from_directory(dir),
    }
}

fn handle_config_action(action: &Option<ConfigAction>) -> Result<()> {
    match action {
        Some(ConfigAction::View) => config::view_config(),
        Some(ConfigAction::Set(key, value)) => config::update_config(key, value),
        None => config::view_config(),
    }
}
