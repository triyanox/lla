use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Clone, Serialize, Deserialize)]
pub struct DecoratedEntry {
    pub path: PathBuf,
    pub metadata: EntryMetadata,
    pub custom_fields: HashMap<String, String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct EntryMetadata {
    pub size: u64,
    pub modified: u64,
    pub accessed: u64,
    pub created: u64,
    pub is_dir: bool,
    pub is_file: bool,
    pub is_symlink: bool,
    pub permissions: u32,
    pub uid: u32,
    pub gid: u32,
}

#[derive(Serialize, Deserialize)]
pub enum PluginRequest {
    GetName,
    GetVersion,
    GetDescription,
    GetSupportedFormats,
    Decorate(DecoratedEntry),
    FormatField(DecoratedEntry, String),
    PerformAction(String, Vec<String>),
}

#[derive(Serialize, Deserialize)]
pub enum PluginResponse {
    Name(String),
    Version(String),
    Description(String),
    SupportedFormats(Vec<String>),
    Decorated(DecoratedEntry),
    FormattedField(Option<String>),
    ActionResult(Result<(), String>),
    Error(String),
}

pub trait Plugin: Send + Sync {
    fn handle_request(&mut self, request: PluginRequest) -> PluginResponse;
}

#[macro_export]
macro_rules! declare_plugin {
    ($plugin_type:ty) => {
        #[no_mangle]
        pub extern "C" fn _plugin_create() -> *mut dyn $crate::Plugin {
            Box::into_raw(Box::new(<$plugin_type>::new()))
        }
    };
}
