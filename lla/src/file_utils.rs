use crate::args::Args;
use crate::config::Config;
use crate::error::Result;
use crate::filter::{
    CaseInsensitiveFilter, CompositeFilter, ExtensionFilter, FileFilter, FilterOperation,
    GlobFilter, PatternFilter, RegexFilter,
};
use crate::formatter::{
    DefaultFormatter, FileFormatter, GitFormatter, GridFormatter, LongFormatter, SizeMapFormatter,
    TableFormatter, TimelineFormatter, TreeFormatter,
};
use crate::lister::{BasicLister, FileLister, RecursiveLister};
use crate::plugin::PluginManager;
use crate::sorter::{AlphabeticalSorter, DateSorter, FileSorter, SizeSorter, SortOptions};
use lla_plugin_interface::proto::{DecoratedEntry, EntryMetadata};
use rayon::prelude::*;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::UNIX_EPOCH;

pub fn list_directory(
    args: &Args,
    plugin_manager: &mut PluginManager,
    config_error: Option<crate::error::LlaError>,
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
        sort_files(decorated_files, &sorter, args)?
    } else {
        decorated_files
    };

    let formatted_output =
        formatter.format_files(decorated_files.as_slice(), plugin_manager, args.depth)?;
    println!("{}", formatted_output);
    Ok(())
}

pub fn get_format(args: &Args) -> &'static str {
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

pub fn convert_metadata(metadata: &std::fs::Metadata) -> EntryMetadata {
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

pub fn list_and_decorate_files(
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

pub fn sort_files(
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

pub fn create_lister(args: &Args) -> Arc<dyn FileLister + Send + Sync> {
    if args.tree_format {
        let config = Config::load(&Config::get_config_path()).unwrap_or_default();
        Arc::new(RecursiveLister::new(config))
    } else {
        Arc::new(BasicLister)
    }
}

pub fn create_sorter(args: &Args) -> Arc<dyn FileSorter + Send + Sync> {
    let sorter: Arc<dyn FileSorter + Send + Sync> = match args.sort_by.as_str() {
        "name" => Arc::new(AlphabeticalSorter),
        "size" => Arc::new(SizeSorter),
        "date" => Arc::new(DateSorter),
        _ => Arc::new(AlphabeticalSorter),
    };

    sorter
}

pub fn create_filter(args: &Args) -> Arc<dyn FileFilter + Send + Sync> {
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

pub fn create_formatter(args: &Args) -> Box<dyn FileFormatter> {
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