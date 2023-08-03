mod args;
mod error;
mod filter;
mod format;
mod ls;
mod sort;

use args::{parse_args, FilterBy, SortBy};
use filter::FileFilter;
use format::{DefaultFileFormatter, TreeFileFormatter};
use ls::FileLister;
use sort::FileSorter;

use crate::{filter::NoFilter, format::FileFormatter};

fn main() {
    let options = parse_args();

    let lister: Box<dyn FileLister>;
    if options.recursive {
        lister = Box::new(ls::TreeLister::new());
    } else {
        match options.long_format {
            true => lister = Box::new(ls::LongLister::new()),
            false => lister = Box::new(ls::BasicLister::new()),
        }
    }

    let sorter: Box<dyn FileSorter> = match options.sort_by {
        SortBy::Name => Box::new(sort::AlphabeticalSorter::new()),
        SortBy::Size => Box::new(sort::SizeSorter::new()),
        SortBy::Date => Box::new(sort::DateSorter::new()),
    };

    let filter: Box<dyn FileFilter> = match options.filter {
        Some(FilterBy::Extension(ext)) => Box::new(filter::ExtensionFilter::new(&ext)),
        Some(FilterBy::Pattern(pattern)) => Box::new(filter::PatternFilter::new(&pattern)),
        None => Box::new(NoFilter::new()),
    };

    let result = lister.list_files(&options.directory, Some(options.recursive), options.depth);
    match result {
        Err(e) => eprintln!("{}", e),
        Ok(files) => {
            let sorted_files = sorter.sort_files(files);
            let filtered_files = filter.filter_files(sorted_files);
            let formatter: Box<dyn FileFormatter> = match options.recursive {
                true => Box::new(TreeFileFormatter::new()),
                false => Box::new(DefaultFileFormatter::new()),
            };
            formatter.display_files(&filtered_files, Some(options.long_format));
        }
    }
}
