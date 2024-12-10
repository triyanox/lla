mod args;
mod config;
mod error;
mod filter;
mod formatter;
mod installer;
mod lister;
mod plugin;
mod sorter;
mod utils;

use args::{Args, Command, ConfigAction, InstallSource};
use colored::*;
use config::{initialize_config, Config};
use dialoguer::{theme::ColorfulTheme, MultiSelect};
use error::{LlaError, Result};
use filter::{ExtensionFilter, FileFilter, PatternFilter};
use formatter::{
    DefaultFormatter, FileFormatter, GitFormatter, GridFormatter, LongFormatter, SizeMapFormatter,
    TableFormatter, TimelineFormatter, TreeFormatter,
};
use installer::PluginInstaller;
use lister::{BasicLister, FileLister, RecursiveLister};
use lla_plugin_interface::{DecoratedEntry, EntryMetadata};
use plugin::PluginManager;
use rayon::prelude::*;
use sorter::{AlphabeticalSorter, DateSorter, FileSorter, SizeSorter};
use std::collections::HashSet;
use std::os::unix::fs::MetadataExt;
use std::sync::Arc;
use std::time::UNIX_EPOCH;

fn main() -> Result<()> {
    let (config, config_error) = load_config()?;
    let args = Args::parse(&config);
    let mut plugin_manager = initialize_plugin_manager(&args, &config)?;

    match args.command {
        Some(Command::Install(source)) => {
            let installer = PluginInstaller::new(&args.plugins_dir);
            match source {
                InstallSource::GitHub(url) => installer.install_from_git(&url),
                InstallSource::LocalDir(dir) => installer.install_from_directory(&dir),
            }
        }
        Some(Command::Update(plugin_name)) => {
            let installer = PluginInstaller::new(&args.plugins_dir);
            installer.update_plugins(plugin_name.as_deref())
        }
        Some(Command::ListPlugins) => list_plugins(&mut plugin_manager),
        Some(Command::Use) => list_plugins(&mut plugin_manager),
        Some(Command::InitConfig) => initialize_config(),
        Some(Command::Config(action)) => match action {
            Some(ConfigAction::View) => config::view_config(),
            Some(ConfigAction::Set(key, value)) => config::update_config(&key, &value),
            None => config::view_config(),
        },
        Some(Command::PluginAction(plugin_name, action, action_args)) => {
            plugin_manager.perform_plugin_action(&plugin_name, &action, &action_args)
        }
        None => list_directory(&args, &mut plugin_manager, config_error),
    }
}

fn list_directory(
    args: &Args,
    plugin_manager: &mut PluginManager,
    config_error: Option<LlaError>,
) -> Result<()> {
    if let Some(error) = config_error {
        eprintln!("Warning: {}", error);
    }

    for plugin in &args.enable_plugin {
        if let Err(e) = plugin_manager.enable_plugin(plugin) {
            eprintln!("Failed to enable plugin '{}': {}", plugin, e);
        }
    }
    for plugin in &args.disable_plugin {
        if let Err(e) = plugin_manager.disable_plugin(plugin) {
            eprintln!("Failed to disable plugin '{}': {}", plugin, e);
        }
    }

    let lister = create_lister(args);
    let sorter = create_sorter(args);
    let filter = create_filter(args);
    let formatter = create_formatter(args);
    let format = get_format(args);

    let decorated_files = list_and_decorate_files(args, &lister, &filter, plugin_manager, format)?;

    let decorated_files = if !args.tree_format {
        sort_files(decorated_files, &sorter)?
    } else {
        decorated_files
    };

    let formatted_output =
        formatter.format_files(decorated_files.as_slice(), plugin_manager, args.depth)?;
    println!("{}", formatted_output);
    Ok(())
}

fn get_format(args: &Args) -> &'static str {
    if args.long_format {
        "long"
    } else if args.tree_format {
        "tree"
    } else if args.table_format {
        "table"
    } else if args.grid_format {
        "grid"
    } else {
        "default"
    }
}

fn convert_metadata(metadata: &std::fs::Metadata) -> EntryMetadata {
    EntryMetadata {
        size: metadata.len(),
        modified: metadata
            .modified()
            .map(|t| t.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs())
            .unwrap_or(0),
        accessed: metadata
            .accessed()
            .map(|t| t.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs())
            .unwrap_or(0),
        created: metadata
            .created()
            .map(|t| t.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs())
            .unwrap_or(0),
        is_dir: metadata.is_dir(),
        is_file: metadata.is_file(),
        is_symlink: metadata.is_symlink(),
        permissions: metadata.mode(),
        uid: metadata.uid(),
        gid: metadata.gid(),
    }
}

fn list_and_decorate_files(
    args: &Args,
    lister: &Arc<dyn FileLister + Send + Sync>,
    filter: &Arc<dyn FileFilter + Send + Sync>,
    plugin_manager: &mut PluginManager,
    format: &str,
) -> Result<Vec<DecoratedEntry>> {
    let mut entries: Vec<DecoratedEntry> = lister
        .list_files(&args.directory, args.tree_format, args.depth)?
        .into_par_iter()
        .filter_map(|path| {
            let fs_metadata = path.metadata().ok()?;

            if !filter
                .filter_files(std::slice::from_ref(&path))
                .map(|v| !v.is_empty())
                .unwrap_or(false)
            {
                return None;
            }

            Some(DecoratedEntry {
                path,
                metadata: convert_metadata(&fs_metadata),
                custom_fields: std::collections::HashMap::with_capacity(8),
            })
        })
        .collect();

    for entry in &mut entries {
        plugin_manager.decorate_entry(entry, format);
    }

    Ok(entries)
}

fn sort_files(
    mut files: Vec<DecoratedEntry>,
    sorter: &Arc<dyn FileSorter + Send + Sync>,
) -> Result<Vec<DecoratedEntry>> {
    let mut paths: Vec<_> = files.iter().map(|entry| entry.path.clone()).collect();
    sorter.sort_files(&mut paths)?;

    files.sort_by_key(|entry| {
        paths
            .iter()
            .position(|p| p == &entry.path)
            .unwrap_or(usize::MAX)
    });

    Ok(files)
}

fn load_config() -> Result<(Config, Option<LlaError>)> {
    match Config::load(&Config::get_config_path()) {
        Ok(config) => Ok((config, None)),
        Err(e) => {
            let error = LlaError::Config(format!("Failed to load config: {}", e));
            Ok((Config::default(), Some(error)))
        }
    }
}

fn initialize_plugin_manager(args: &Args, config: &Config) -> Result<PluginManager> {
    let mut plugin_manager = PluginManager::new(config.clone());
    plugin_manager.discover_plugins(&args.plugins_dir)?;
    Ok(plugin_manager)
}

fn list_plugins(plugin_manager: &mut PluginManager) -> Result<()> {
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

fn create_lister(args: &Args) -> Arc<dyn FileLister + Send + Sync> {
    if args.tree_format {
        let config = Config::load(&Config::get_config_path()).unwrap_or_default();
        Arc::new(RecursiveLister::new(config))
    } else {
        Arc::new(BasicLister)
    }
}

fn create_sorter(args: &Args) -> Arc<dyn FileSorter + Send + Sync> {
    match args.sort_by.as_str() {
        "name" => Arc::new(AlphabeticalSorter),
        "size" => Arc::new(SizeSorter),
        "date" => Arc::new(DateSorter),
        _ => Arc::new(AlphabeticalSorter),
    }
}

fn create_filter(args: &Args) -> Arc<dyn FileFilter + Send + Sync> {
    match &args.filter {
        Some(filter_str) if filter_str.starts_with('.') => {
            Arc::new(ExtensionFilter::new(filter_str[1..].to_string()))
        }
        Some(filter_str) => Arc::new(PatternFilter::new(filter_str.clone())),
        None => Arc::new(PatternFilter::new("".to_string())),
    }
}

fn create_formatter(args: &Args) -> Arc<dyn FileFormatter + Send + Sync> {
    if args.long_format {
        Arc::new(LongFormatter)
    } else if args.tree_format {
        Arc::new(TreeFormatter)
    } else if args.table_format {
        Arc::new(TableFormatter)
    } else if args.grid_format {
        Arc::new(GridFormatter)
    } else if args.sizemap_format {
        Arc::new(SizeMapFormatter)
    } else if args.timeline_format {
        Arc::new(TimelineFormatter)
    } else if args.git_format {
        Arc::new(GitFormatter)
    } else {
        Arc::new(DefaultFormatter)
    }
}
