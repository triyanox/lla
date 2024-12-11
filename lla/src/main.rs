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

use args::{Args, Command, ConfigAction, InstallSource, ShortcutAction};
use colored::*;
use config::{initialize_config, Config};
use dialoguer::{theme::ColorfulTheme, MultiSelect};
use error::{LlaError, Result};
use filter::{
    CaseInsensitiveFilter, CompositeFilter, ExtensionFilter, FileFilter, FilterOperation,
    GlobFilter, PatternFilter, RegexFilter,
};
use formatter::{
    DefaultFormatter, FileFormatter, GitFormatter, GridFormatter, LongFormatter, SizeMapFormatter,
    TableFormatter, TimelineFormatter, TreeFormatter,
};
use installer::PluginInstaller;
use lister::{BasicLister, FileLister, RecursiveLister};
use lla_plugin_interface::proto::{DecoratedEntry, EntryMetadata};
use plugin::PluginManager;
use rayon::prelude::*;
use sorter::{AlphabeticalSorter, DateSorter, FileSorter, SizeSorter, SortOptions};
use std::collections::HashSet;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::UNIX_EPOCH;

fn main() -> Result<()> {
    let (mut config, config_error) = load_config()?;
    let args = Args::parse(&config);

    if let Some(Command::Clean) = args.command {
        println!("🔄 Starting plugin cleaning...");
        let mut plugin_manager = PluginManager::new(config.clone());
        return plugin_manager.clean_plugins();
    }

    let mut plugin_manager = initialize_plugin_manager(&args, &config)?;

    match args.command {
        Some(Command::Shortcut(action)) => match action {
            ShortcutAction::Add(name, command) => {
                config.add_shortcut(name.clone(), command.clone())?;
                println!(
                    "✓ Added shortcut '{}' -> {} {}",
                    name.green(),
                    command.plugin_name.cyan(),
                    command.action.cyan()
                );
                if let Some(desc) = command.description {
                    println!("  Description: {}", desc);
                }
                Ok(())
            }
            ShortcutAction::Remove(name) => {
                if config.get_shortcut(&name).is_some() {
                    config.remove_shortcut(&name)?;
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
            ShortcutAction::Run(name, args) => match config.get_shortcut(&name) {
                Some(shortcut) => {
                    let plugin_name = shortcut.plugin_name.clone();
                    let action = shortcut.action.clone();
                    handle_plugin_action(&mut config, &plugin_name, &action, &args)
                }
                None => {
                    println!("✗ Shortcut '{}' not found", name.red());
                    Ok(())
                }
            },
        },
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
        Some(Command::Clean) => unreachable!(),
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
        sort_files(decorated_files, &sorter, &args)?
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
                path: path.to_string_lossy().into_owned(),
                metadata: Some(convert_metadata(&fs_metadata)),
                custom_fields: Default::default(),
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
    args: &Args,
) -> Result<Vec<DecoratedEntry>> {
    let mut paths: Vec<PathBuf> = files
        .iter()
        .map(|entry| PathBuf::from(&entry.path))
        .collect();

    let options = SortOptions {
        reverse: args.sort_reverse,
        dirs_first: args.sort_dirs_first,
        case_sensitive: args.sort_case_sensitive,
        natural: args.sort_natural,
    };

    sorter.sort_files(&mut paths, options)?;

    files.sort_by_key(|entry| {
        paths
            .iter()
            .position(|p| p == &PathBuf::from(&entry.path))
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
    let sorter: Arc<dyn FileSorter + Send + Sync> = match args.sort_by.as_str() {
        "name" => Arc::new(AlphabeticalSorter),
        "size" => Arc::new(SizeSorter),
        "date" => Arc::new(DateSorter),
        _ => Arc::new(AlphabeticalSorter),
    };

    sorter
}

fn create_filter(args: &Args) -> Arc<dyn FileFilter + Send + Sync> {
    match &args.filter {
        Some(filter_str) => {
            if filter_str.contains(" AND ") {
                let mut composite = CompositeFilter::new(FilterOperation::And);
                for part in filter_str.split(" AND ") {
                    composite.add_filter(create_base_filter(part.trim(), !args.case_sensitive));
                }
                Arc::new(composite)
            } else if filter_str.contains(" OR ") {
                let mut composite = CompositeFilter::new(FilterOperation::Or);
                for part in filter_str.split(" OR ") {
                    composite.add_filter(create_base_filter(part.trim(), !args.case_sensitive));
                }
                Arc::new(composite)
            } else if filter_str.starts_with("NOT ") {
                let mut composite = CompositeFilter::new(FilterOperation::Not);
                composite.add_filter(create_base_filter(&filter_str[4..], !args.case_sensitive));
                Arc::new(composite)
            } else if filter_str.starts_with("XOR ") {
                let mut composite = CompositeFilter::new(FilterOperation::Xor);
                composite.add_filter(create_base_filter(&filter_str[4..], !args.case_sensitive));
                Arc::new(composite)
            } else {
                Arc::from(create_base_filter(filter_str, !args.case_sensitive))
            }
        }
        None => Arc::new(PatternFilter::new("".to_string())),
    }
}

fn create_base_filter(pattern: &str, case_insensitive: bool) -> Box<dyn FileFilter + Send + Sync> {
    let base_filter: Box<dyn FileFilter + Send + Sync> = if pattern.starts_with("regex:") {
        Box::new(RegexFilter::new(pattern[6..].to_string()))
    } else if pattern.starts_with("glob:") {
        Box::new(GlobFilter::new(pattern[5..].to_string()))
    } else if pattern.starts_with('.') {
        Box::new(ExtensionFilter::new(pattern[1..].to_string()))
    } else {
        Box::new(PatternFilter::new(pattern.to_string()))
    };

    if case_insensitive {
        Box::new(CaseInsensitiveFilter::new(base_filter))
    } else {
        base_filter
    }
}

fn create_formatter(args: &Args) -> Box<dyn FileFormatter> {
    if args.long_format {
        Box::new(LongFormatter::new(args.show_icons))
    } else if args.tree_format {
        Box::new(TreeFormatter::new(args.show_icons))
    } else if args.table_format {
        Box::new(TableFormatter::new(args.show_icons))
    } else if args.grid_format {
        Box::new(GridFormatter::new(args.show_icons))
    } else if args.sizemap_format {
        Box::new(SizeMapFormatter::new(args.show_icons))
    } else if args.timeline_format {
        Box::new(TimelineFormatter::new(args.show_icons))
    } else if args.git_format {
        Box::new(GitFormatter::new(args.show_icons))
    } else {
        Box::new(DefaultFormatter::new(args.show_icons))
    }
}

fn handle_plugin_action(
    config: &mut Config,
    plugin_name: &str,
    action: &str,
    args: &[String],
) -> Result<()> {
    let mut plugin_manager = PluginManager::new(config.clone());
    plugin_manager.discover_plugins(&config.plugins_dir)?;
    plugin_manager.perform_plugin_action(plugin_name, action, args)
}
