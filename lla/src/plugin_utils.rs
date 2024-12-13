use crate::config::Config;
use crate::error::Result;
use crate::plugin::PluginManager;
use colored::*;
use dialoguer::{theme::ColorfulTheme, MultiSelect};
use std::collections::HashSet;

pub fn list_plugins(plugin_manager: &mut PluginManager) -> Result<()> {
    if atty::is(atty::Stream::Stdout) {
        let plugins: Vec<(String, String, String)> = plugin_manager
            .list_plugins()
            .into_iter()
            .map(|(name, version, desc)| (name, version, desc))
            .collect();

        let plugin_names: Vec<String> = plugins
            .iter()
            .map(|(name, version, desc)| {
                format!(
                    "{} {} - {}",
                    name.cyan(),
                    format!("v{}", version).yellow(),
                    desc
                )
            })
            .collect();

        println!("\n{}", "Plugin Manager".cyan().bold());
        println!("{}\n", "Space to toggle, Enter to confirm".bright_black());

        let theme = ColorfulTheme {
            active_item_style: dialoguer::console::Style::new().cyan().bold(),
            active_item_prefix: dialoguer::console::style("│ ⦿ ".to_string())
                .for_stderr()
                .cyan(),
            checked_item_prefix: dialoguer::console::style("  ◉ ".to_string())
                .for_stderr()
                .green(),
            unchecked_item_prefix: dialoguer::console::style("  ○ ".to_string())
                .for_stderr()
                .red(),
            prompt_prefix: dialoguer::console::style("│ ".to_string())
                .for_stderr()
                .cyan(),
            prompt_style: dialoguer::console::Style::new().for_stderr().cyan(),
            success_prefix: dialoguer::console::style("│ ".to_string())
                .for_stderr()
                .cyan(),
            ..ColorfulTheme::default()
        };

        let selections = MultiSelect::with_theme(&theme)
            .with_prompt("Select plugins")
            .items(&plugin_names)
            .defaults(
                &plugins
                    .iter()
                    .map(|(name, _, _)| plugin_manager.enabled_plugins.contains(name))
                    .collect::<Vec<_>>(),
            )
            .interact()?;

        let mut updated_plugins = HashSet::new();

        for idx in selections {
            let (name, _, _) = &plugins[idx];
            updated_plugins.insert(name.to_string());
        }

        for (name, _, _) in &plugins {
            if updated_plugins.contains(name) {
                plugin_manager.enable_plugin(name)?;
            } else {
                plugin_manager.disable_plugin(name)?;
            }
        }
    } else {
        for (name, version, desc) in plugin_manager.list_plugins() {
            println!(
                "{} {} - {}",
                name.cyan(),
                format!("v{}", version).yellow(),
                desc
            );
        }
    }

    Ok(())
}

pub fn handle_plugin_action(
    config: &mut Config,
    plugin_name: &str,
    action: &str,
    args: &[String],
) -> Result<()> {
    let mut plugin_manager = PluginManager::new(config.clone());
    plugin_manager.discover_plugins(&config.plugins_dir)?;
    plugin_manager.perform_plugin_action(plugin_name, action, args)
}
