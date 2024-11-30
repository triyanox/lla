use crate::error::Result;
use crate::plugin::PluginManager;
use lla_plugin_interface::DecoratedEntry;

pub trait FileFormatter {
    fn format_files(
        &self,
        files: &[DecoratedEntry],
        plugin_manager: &PluginManager,
        depth: Option<usize>,
    ) -> Result<String>;
}

mod default;
mod git;
mod grid;
mod long;
mod sizemap;
mod table;
mod timeline;
mod tree;

pub use default::DefaultFormatter;
pub use git::GitFormatter;
pub use grid::GridFormatter;
pub use long::LongFormatter;
pub use sizemap::SizeMapFormatter;
pub use table::TableFormatter;
pub use timeline::TimelineFormatter;
pub use tree::TreeFormatter;
