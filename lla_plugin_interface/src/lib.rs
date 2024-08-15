use std::collections::HashMap;
use std::path::PathBuf;

pub struct DecoratedEntry {
    pub path: PathBuf,
    pub metadata: std::fs::Metadata,
    pub custom_fields: HashMap<String, String>,
}

pub struct CliArg {
    pub name: String,
    pub short: Option<char>,
    pub long: Option<String>,
    pub help: String,
    pub takes_value: bool,
}

pub trait EntryDecorator: Send + Sync {
    fn name(&self) -> &'static str;
    fn decorate(&self, entry: &mut DecoratedEntry);
    fn supported_formats(&self) -> Vec<&'static str> {
        vec!["default", "long", "tree"]
    }
    fn format_field(&self, entry: &DecoratedEntry, format: &str) -> Option<String>;
}

pub trait Plugin: EntryDecorator {
    fn version(&self) -> &'static str;
    fn description(&self) -> &'static str;

    fn cli_args(&self) -> Vec<CliArg> {
        Vec::new()
    }
    fn handle_cli_args(&self, _args: &[String]) {}
    fn perform_action(&self, _action: &str, _args: &[String]) -> Result<(), String> {
        Ok(())
    }
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
