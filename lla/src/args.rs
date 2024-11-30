use crate::config::Config;
use clap::{App, Arg, ArgMatches, SubCommand};
use std::path::PathBuf;

pub struct Args {
    pub directory: String,
    pub depth: Option<usize>,
    pub long_format: bool,
    pub tree_format: bool,
    pub table_format: bool,
    pub grid_format: bool,
    pub sizemap_format: bool,
    pub sort_by: String,
    pub filter: Option<String>,
    pub enable_plugin: Vec<String>,
    pub disable_plugin: Vec<String>,
    pub plugins_dir: PathBuf,
    pub command: Option<Command>,
    pub plugin_args: Vec<String>,
}

pub enum Command {
    Install(InstallSource),
    ListPlugins,
    InitConfig,
    Config(Option<ConfigAction>),
    PluginAction(String, String, Vec<String>),
}

pub enum InstallSource {
    GitHub(String),
    LocalDir(String),
}

pub enum ConfigAction {
    View,
    Set(String, String),
}

impl Args {
    pub fn parse(config: &Config) -> Self {
        let matches = App::new(env!("CARGO_PKG_NAME"))
            .version(env!("CARGO_PKG_VERSION"))
            .author(env!("CARGO_PKG_AUTHORS"))
            .about(env!("CARGO_PKG_DESCRIPTION"))
            .arg(
                Arg::with_name("directory")
                    .help("The directory to list")
                    .index(1)
                    .default_value("."),
            )
            .arg(
                Arg::with_name("depth")
                    .short('d')
                    .long("depth")
                    .takes_value(true)
                    .help("Set the depth for tree listing"),
            )
            .arg(
                Arg::with_name("long")
                    .short('l')
                    .long("long")
                    .help("Use long listing format"),
            )
            .arg(
                Arg::with_name("tree")
                    .short('t')
                    .long("tree")
                    .help("Use tree listing format"),
            )
            .arg(
                Arg::with_name("table")
                    .short('T')
                    .long("table")
                    .help("Use table listing format"),
            )
            .arg(
                Arg::with_name("grid")
                    .short('g')
                    .long("grid")
                    .help("Use grid listing format"),
            )
            .arg(
                Arg::with_name("sizemap")
                    .short('S')
                    .long("sizemap")
                    .help("Show visual representation of file sizes"),
            )
            .arg(
                Arg::with_name("sort")
                    .short('s')
                    .long("sort")
                    .takes_value(true)
                    .possible_values(["name", "size", "date"])
                    .default_value(&config.default_sort),
            )
            .arg(
                Arg::with_name("filter")
                    .short('f')
                    .long("filter")
                    .takes_value(true)
                    .help("Filter files by name or extension"),
            )
            .arg(
                Arg::with_name("enable-plugin")
                    .long("enable-plugin")
                    .takes_value(true)
                    .multiple(true)
                    .help("Enable specific plugins"),
            )
            .arg(
                Arg::with_name("disable-plugin")
                    .long("disable-plugin")
                    .takes_value(true)
                    .multiple(true)
                    .help("Disable specific plugins"),
            )
            .arg(
                Arg::with_name("plugins-dir")
                    .long("plugins-dir")
                    .takes_value(true)
                    .help("Specify the plugins directory"),
            )
            .subcommand(
                SubCommand::with_name("install")
                    .about("Install a plugin")
                    .arg(
                        Arg::with_name("git")
                            .long("git")
                            .takes_value(true)
                            .help("Install a plugin from a GitHub repository URL"),
                    )
                    .arg(
                        Arg::with_name("dir")
                            .long("dir")
                            .takes_value(true)
                            .help("Install a plugin from a local directory"),
                    ),
            )
            .arg(
                Arg::with_name("plugin-arg")
                    .long("plugin-arg")
                    .takes_value(true)
                    .multiple(true)
                    .help("Arguments to pass to enabled plugins"),
            )
            .subcommand(
                SubCommand::with_name("plugin")
                    .about("Run a plugin action")
                    .arg(
                        Arg::with_name("name")
                            .long("name")
                            .short('n')
                            .takes_value(true)
                            .required(true)
                            .help("Name of the plugin"),
                    )
                    .arg(
                        Arg::with_name("action")
                            .long("action")
                            .short('a')
                            .takes_value(true)
                            .required(true)
                            .help("Action to perform"),
                    )
                    .arg(
                        Arg::with_name("args")
                            .long("args")
                            .short('r')
                            .takes_value(true)
                            .multiple(true)
                            .help("Arguments for the plugin action"),
                    ),
            )
            .subcommand(SubCommand::with_name("list-plugins").about("List all available plugins"))
            .subcommand(SubCommand::with_name("init").about("Initialize the configuration file"))
            .subcommand(
                SubCommand::with_name("config")
                    .about("View or modify configuration")
                    .arg(
                        Arg::with_name("set")
                            .long("set")
                            .takes_value(true)
                            .number_of_values(2)
                            .value_names(&["KEY", "VALUE"])
                            .help("Set a configuration value (e.g., --set plugins_dir /new/path)"),
                    ),
            )
            .get_matches();

        Self::from_matches(&matches, config)
    }

    fn from_matches(matches: &ArgMatches, config: &Config) -> Self {
        let format = if matches.is_present("long") {
            "long"
        } else if matches.is_present("tree") {
            "tree"
        } else if matches.is_present("table") {
            "table"
        } else if matches.is_present("grid") {
            "grid"
        } else if matches.is_present("sizemap") {
            "sizemap"
        } else {
            &config.default_format
        };

        let command = if let Some(install_matches) = matches.subcommand_matches("install") {
            if let Some(github_url) = install_matches.value_of("git") {
                Some(Command::Install(InstallSource::GitHub(
                    github_url.to_string(),
                )))
            } else {
                install_matches.value_of("dir").map(|local_dir| {
                    Command::Install(InstallSource::LocalDir(local_dir.to_string()))
                })
            }
        } else if matches.subcommand_matches("list-plugins").is_some() {
            Some(Command::ListPlugins)
        } else if matches.subcommand_matches("init").is_some() {
            Some(Command::InitConfig)
        } else if let Some(config_matches) = matches.subcommand_matches("config") {
            if let Some(values) = config_matches.values_of("set") {
                let values: Vec<_> = values.collect();
                Some(Command::Config(Some(ConfigAction::Set(
                    values[0].to_string(),
                    values[1].to_string(),
                ))))
            } else {
                Some(Command::Config(Some(ConfigAction::View)))
            }
        } else if let Some(plugin_matches) = matches.subcommand_matches("plugin") {
            let plugin_name = plugin_matches.value_of("name").unwrap().to_string();
            let action = plugin_matches.value_of("action").unwrap().to_string();
            let args = plugin_matches
                .values_of("args")
                .map(|v| v.map(String::from).collect())
                .unwrap_or_default();
            Some(Command::PluginAction(plugin_name, action, args))
        } else {
            None
        };

        Args {
            directory: matches.value_of("directory").unwrap().to_string(),
            depth: matches
                .value_of("depth")
                .map(|d| d.parse().unwrap())
                .or(config.default_depth),
            long_format: format == "long",
            tree_format: format == "tree",
            table_format: format == "table",
            grid_format: format == "grid",
            sizemap_format: format == "sizemap",
            sort_by: matches
                .value_of("sort")
                .unwrap_or(&config.default_sort)
                .to_string(),
            filter: matches.value_of("filter").map(String::from),
            enable_plugin: matches
                .values_of("enable-plugin")
                .map(|v| v.map(String::from).collect())
                .unwrap_or_else(|| config.enabled_plugins.clone()),
            disable_plugin: matches
                .values_of("disable-plugin")
                .map(|v| v.map(String::from).collect())
                .unwrap_or_default(),
            plugins_dir: matches
                .value_of("plugins-dir")
                .map(PathBuf::from)
                .unwrap_or_else(|| config.plugins_dir.clone()),
            command,
            plugin_args: matches
                .values_of("plugin-arg")
                .map(|v| v.map(String::from).collect())
                .unwrap_or_default(),
        }
    }
}
