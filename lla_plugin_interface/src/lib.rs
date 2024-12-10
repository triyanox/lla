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
    fn handle_request(&mut self, request: PluginRequest) -> PluginResponse;

    fn handle_raw_request(&mut self, request: &[u8]) -> Vec<u8> {
        use prost::Message;

        let proto_msg = match proto::PluginMessage::decode(request) {
            Ok(msg) => msg,
            Err(e) => return encode_error(&format!("Failed to decode request: {}", e)),
        };

        let request = match proto_to_request(proto_msg) {
            Ok(req) => req,
            Err(e) => return encode_error(&format!("Failed to convert request: {}", e)),
        };

        let response = self.handle_request(request);

        let proto_response = response_to_proto(response);
        let mut buf = BytesMut::with_capacity(proto_response.encoded_len());
        proto_response.encode(&mut buf).unwrap();
        buf.to_vec()
    }
}

fn proto_to_request(msg: proto::PluginMessage) -> Result<PluginRequest, std::io::Error> {
    use proto::plugin_message::Message;

    match msg.message {
        Some(Message::GetName(_)) => Ok(PluginRequest::GetName),
        Some(Message::GetVersion(_)) => Ok(PluginRequest::GetVersion),
        Some(Message::GetDescription(_)) => Ok(PluginRequest::GetDescription),
        Some(Message::GetSupportedFormats(_)) => Ok(PluginRequest::GetSupportedFormats),
        Some(Message::Decorate(entry)) => Ok(PluginRequest::Decorate(entry.try_into()?)),
        Some(Message::FormatField(req)) => Ok(PluginRequest::FormatField(
            req.entry.unwrap_or_default().try_into()?,
            req.format,
        )),
        Some(Message::Action(req)) => Ok(PluginRequest::PerformAction(req.action, req.args)),
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid request type",
        )),
    }
}

fn response_to_proto(response: PluginResponse) -> proto::PluginMessage {
    use proto::plugin_message::Message;

    let message = match response {
        PluginResponse::Name(name) => Message::NameResponse(name),
        PluginResponse::Version(version) => Message::VersionResponse(version),
        PluginResponse::Description(desc) => Message::DescriptionResponse(desc),
        PluginResponse::SupportedFormats(formats) => {
            Message::FormatsResponse(proto::SupportedFormatsResponse { formats })
        }
        PluginResponse::Decorated(entry) => Message::DecoratedResponse(entry.into()),
        PluginResponse::FormattedField(field) => {
            Message::FieldResponse(proto::FormattedFieldResponse { field })
        }
        PluginResponse::ActionResult(result) => Message::ActionResponse(proto::ActionResponse {
            success: result.is_ok(),
            error: result.err(),
        }),
        PluginResponse::Error(error) => Message::ErrorResponse(error),
    };

    proto::PluginMessage {
        message: Some(message),
    }
}

fn encode_error(error: &str) -> Vec<u8> {
    use prost::Message;
    let response = proto::PluginMessage {
        message: Some(proto::plugin_message::Message::ErrorResponse(
            error.to_string(),
        )),
    };
    let mut buf = BytesMut::with_capacity(response.encoded_len());
    response.encode(&mut buf).unwrap();
    buf.to_vec()
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
