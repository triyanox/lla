use bytes::BytesMut;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/lla_plugin.rs"));
}

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

impl From<EntryMetadata> for proto::EntryMetadata {
    fn from(meta: EntryMetadata) -> Self {
        proto::EntryMetadata {
            size: meta.size,
            modified: meta.modified,
            accessed: meta.accessed,
            created: meta.created,
            is_dir: meta.is_dir,
            is_file: meta.is_file,
            is_symlink: meta.is_symlink,
            permissions: meta.permissions,
            uid: meta.uid,
            gid: meta.gid,
        }
    }
}

impl From<proto::EntryMetadata> for EntryMetadata {
    fn from(meta: proto::EntryMetadata) -> Self {
        EntryMetadata {
            size: meta.size,
            modified: meta.modified,
            accessed: meta.accessed,
            created: meta.created,
            is_dir: meta.is_dir,
            is_file: meta.is_file,
            is_symlink: meta.is_symlink,
            permissions: meta.permissions,
            uid: meta.uid,
            gid: meta.gid,
        }
    }
}

impl From<DecoratedEntry> for proto::DecoratedEntry {
    fn from(entry: DecoratedEntry) -> Self {
        proto::DecoratedEntry {
            path: entry.path.to_string_lossy().to_string(),
            metadata: Some(entry.metadata.into()),
            custom_fields: entry.custom_fields,
        }
    }
}

impl TryFrom<proto::DecoratedEntry> for DecoratedEntry {
    type Error = std::io::Error;

    fn try_from(entry: proto::DecoratedEntry) -> Result<Self, Self::Error> {
        Ok(DecoratedEntry {
            path: PathBuf::from(entry.path),
            metadata: entry.metadata.unwrap_or_default().into(),
            custom_fields: entry.custom_fields,
        })
    }
}

pub trait Plugin: Send + Sync {
    fn handle_raw_request(&mut self, request: &[u8]) -> Vec<u8>;

    fn handle_request(&mut self, request: PluginRequest) -> PluginResponse {
        use prost::Message;

        let proto_msg = proto::PluginMessage {
            message: Some(match request {
                PluginRequest::GetName => proto::plugin_message::Message::GetName(true),
                PluginRequest::GetVersion => proto::plugin_message::Message::GetVersion(true),
                PluginRequest::GetDescription => {
                    proto::plugin_message::Message::GetDescription(true)
                }
                PluginRequest::GetSupportedFormats => {
                    proto::plugin_message::Message::GetSupportedFormats(true)
                }
                PluginRequest::Decorate(entry) => {
                    proto::plugin_message::Message::Decorate(entry.into())
                }
                PluginRequest::FormatField(entry, format) => {
                    proto::plugin_message::Message::FormatField(proto::FormatFieldRequest {
                        entry: Some(entry.into()),
                        format,
                    })
                }
                PluginRequest::PerformAction(action, args) => {
                    proto::plugin_message::Message::Action(proto::ActionRequest { action, args })
                }
            }),
        };

        let mut buf = BytesMut::with_capacity(proto_msg.encoded_len());
        proto_msg.encode(&mut buf).unwrap();
        let response_bytes = self.handle_raw_request(&buf);

        match proto::PluginMessage::decode(&*response_bytes) {
            Ok(proto_msg) => match proto_msg.message {
                Some(proto::plugin_message::Message::NameResponse(name)) => {
                    PluginResponse::Name(name)
                }
                Some(proto::plugin_message::Message::VersionResponse(version)) => {
                    PluginResponse::Version(version)
                }
                Some(proto::plugin_message::Message::DescriptionResponse(desc)) => {
                    PluginResponse::Description(desc)
                }
                Some(proto::plugin_message::Message::FormatsResponse(formats)) => {
                    PluginResponse::SupportedFormats(formats.formats)
                }
                Some(proto::plugin_message::Message::DecoratedResponse(entry)) => {
                    match DecoratedEntry::try_from(entry) {
                        Ok(entry) => PluginResponse::Decorated(entry),
                        Err(e) => PluginResponse::Error(format!(
                            "Failed to convert decorated entry: {}",
                            e
                        )),
                    }
                }
                Some(proto::plugin_message::Message::FieldResponse(field)) => {
                    PluginResponse::FormattedField(field.field)
                }
                Some(proto::plugin_message::Message::ActionResponse(result)) => {
                    if result.success {
                        PluginResponse::ActionResult(Ok(()))
                    } else {
                        PluginResponse::ActionResult(Err(result
                            .error
                            .unwrap_or_else(|| "Unknown error".to_string())))
                    }
                }
                Some(proto::plugin_message::Message::ErrorResponse(error)) => {
                    PluginResponse::Error(error)
                }
                _ => PluginResponse::Error("Invalid response type".to_string()),
            },
            Err(e) => PluginResponse::Error(format!("Failed to decode response: {}", e)),
        }
    }
}

#[macro_export]
macro_rules! declare_plugin {
    ($plugin_type:ty) => {
        #[no_mangle]
        pub extern "C" fn _plugin_create() -> *mut dyn $crate::Plugin {
            Box::into_raw(Box::new(<$plugin_type>::new()))
        }

        #[no_mangle]
        pub extern "C" fn _plugin_handle_request(request: &[u8]) -> Vec<u8> {
            let mut plugin = unsafe { Box::from_raw(_plugin_create() as *mut $plugin_type) };
            let response = plugin.handle_raw_request(request);
            Box::into_raw(plugin);
            response
        }
    };
}
