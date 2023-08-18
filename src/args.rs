extern crate clap;
use clap::{App, Arg};

#[derive(Debug)]
pub enum SortBy {
    Name,
    Size,
    Date,
}

#[derive(Debug)]
pub enum FilterBy {
    Extension(String),
    Pattern(String),
}

#[derive(Debug)]
pub struct Options {
    pub directory: String,
    pub long_format: bool,
    pub sort_by: SortBy,
    pub filter: Option<FilterBy>,
    pub recursive: bool,
    pub depth: Option<u32>,
    pub git: bool,
}

pub fn parse_args() -> Options {
    let matches = App::new("lla")
        .version(std::env!("CARGO_PKG_VERSION"))
        .author("Mohamed Achaq")
        .about("A lightweight ls replacement")
        .arg(
            Arg::with_name("directory")
                .help("Directory to list")
                .default_value(".")
                .index(1),
        )
        .arg(
            Arg::with_name("long")
                .short('l')
                .long("long")
                .help("Use a long listing format"),
        )
        .arg(
            Arg::with_name("sort")
                .short('s')
                .long("sort")
                .help("Sort by name, size, or date")
                .takes_value(true)
                .possible_values(&["name", "size", "date"]),
        )
        .arg(
            Arg::with_name("filter")
                .short('f')
                .long("filter")
                .help("Filter by extension or pattern")
                .takes_value(true)
                .value_name("FILTER"),
        )
        .arg(
            Arg::with_name("recursive")
                .short('r')
                .long("recursive")
                .help("List subdirectories recursively"),
        )
        .arg(
            Arg::with_name("depth")
                .short('d')
                .long("depth")
                .help("Maximum depth of recursion")
                .takes_value(true)
                .value_name("DEPTH"),
        )
        .arg(
            Arg::with_name("git")
                .short('g')
                .long("git")
                .help("Show git status"),
        )
        .get_matches();

    let directory = matches.value_of("directory").unwrap().to_string();
    let long_format = matches.is_present("long");
    let sort_by = match matches.value_of("sort") {
        Some("name") => SortBy::Name,
        Some("size") => SortBy::Size,
        Some("date") => SortBy::Date,
        _ => SortBy::Name,
    };
    let filter = match matches.value_of("filter") {
        Some(filter) => {
            if filter.starts_with(".") {
                Some(FilterBy::Extension(filter.to_string()))
            } else {
                Some(FilterBy::Pattern(filter.to_string()))
            }
        }
        None => None,
    };
    let recursive = matches.is_present("recursive");
    let depth = match matches.value_of("depth") {
        Some(depth) => match depth.parse::<u32>() {
            Ok(depth) => Some(depth),
            Err(_) => None,
        },
        None => None,
    };
    let git = matches.is_present("git");

    Options {
        directory,
        long_format,
        sort_by,
        filter,
        recursive,
        depth,
        git,
    }
}
