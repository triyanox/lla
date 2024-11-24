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

use args::{Args, Command, InstallSource};
use config::{initialize_config, Config};
use error::{LlaError, Result};
use filter::{ExtensionFilter, FileFilter, PatternFilter};
use formatter::{DefaultFormatter, FileFormatter, LongFormatter, TreeFormatter};
use installer::PluginInstaller;
use lister::{BasicLister, FileLister, RecursiveLister};
use lla_plugin_interface::DecoratedEntry;
use plugin::PluginManager;
use sorter::{AlphabeticalSorter, DateSorter, FileSorter, SizeSorter};

use rayon::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;

fn main() -> Result<()> {
    let (config, config_error) = load_config()?;
    let args = Args::parse(&config);

    let mut plugin_manager = initialize_plugin_manager(&args, &config)?;
    plugin_manager.handle_plugin_args(&args.plugin_args);

    match args.command {
        Some(Command::Install(source)) => {
            let plugin_installer = PluginInstaller::new(&args.plugins_dir);
            match source {
                InstallSource::GitHub(url) => install_plugin_from_git(&plugin_installer, &url),
                InstallSource::LocalDir(dir) => {
                    install_plugin_from_directory(&plugin_installer, &dir)
                }
            }
        }
        Some(Command::ListPlugins) => list_plugins(&plugin_manager),
        Some(Command::InitConfig) => initialize_config(),
        Some(Command::PluginAction(plugin_name, action, action_args)) => {
            plugin_manager.perform_plugin_action(&plugin_name, &action, &action_args)
        }
        Some(Command::Config) => Ok(config::view_config()?),
        None => {
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

            let lister = create_lister(&args);
            let sorter = create_sorter(&args);
            let filter = create_filter(&args);
            let formatter = create_formatter(&args);

            let files = list_files(&args, &lister)?;
            let sorted_files = sort_files(files, &sorter)?;
            let filtered_files = filter_files(sorted_files, &filter);

            let mut decorated_files: Vec<DecoratedEntry> = filtered_files
                .into_iter()
                .filter_map(|path| {
                    path.metadata().ok().map(|metadata| DecoratedEntry {
                        path,
                        metadata,
                        custom_fields: std::collections::HashMap::new(),
                    })
                })
                .collect();

            let format = if args.long_format {
                "long"
            } else if args.tree_format {
                "tree"
            } else {
                "default"
            };

            for entry in &mut decorated_files {
                plugin_manager.decorate_entry(entry, format);
            }

            let formatted_output =
                formatter.format_files(&decorated_files, &plugin_manager, args.depth)?;

            println!("{}", formatted_output);

            Ok(())
        }
    }
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

fn list_plugins(plugin_manager: &PluginManager) -> Result<()> {
    println!("Available plugins:");
    for (name, version, description) in plugin_manager.list_plugins() {
        println!("  {} v{} - {}", name, version, description);
    }
    Ok(())
}

fn create_lister(args: &Args) -> Arc<dyn FileLister + Send + Sync> {
    if args.recursive {
        Arc::new(RecursiveLister)
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
    } else {
        Arc::new(DefaultFormatter)
    }
}

fn list_files(args: &Args, lister: &Arc<dyn FileLister + Send + Sync>) -> Result<Vec<PathBuf>> {
    lister.list_files(&args.directory, args.recursive, args.depth)
}

fn sort_files(
    mut files: Vec<PathBuf>,
    sorter: &Arc<dyn FileSorter + Send + Sync>,
) -> Result<Vec<PathBuf>> {
    sorter.sort_files(&mut files)?;
    Ok(files)
}

fn filter_files(files: Vec<PathBuf>, filter: &Arc<dyn FileFilter + Send + Sync>) -> Vec<PathBuf> {
    files
        .into_par_iter()
        .filter(|file| {
            filter
                .filter_files(&[file.clone()])
                .map(|v| !v.is_empty())
                .unwrap_or(false)
        })
        .collect()
}

fn install_plugin_from_git(installer: &PluginInstaller, url: &str) -> Result<()> {
    installer.install_from_git(url)?;
    println!("Plugin(s) installed successfully from GitHub");
    Ok(())
}

fn install_plugin_from_directory(installer: &PluginInstaller, dir: &str) -> Result<()> {
    installer.install_from_directory(dir)?;
    println!("Plugin(s) installed successfully from local directory");
    Ok(())
}
