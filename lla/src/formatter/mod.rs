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
mod long;
mod tree;
mod table;
mod grid;
mod sizemap;
mod timeline;

pub use default::DefaultFormatter;
pub use long::LongFormatter;
pub use tree::TreeFormatter;
pub use table::TableFormatter;
pub use grid::GridFormatter;
pub use sizemap::SizeMapFormatter;
pub use timeline::TimelineFormatter;
