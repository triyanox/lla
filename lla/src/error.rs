use colored::*;
use std::fmt;
use std::io;

#[allow(dead_code)]
#[derive(Debug)]
pub enum ConfigErrorKind {
    InvalidFormat(String),
    InvalidValue(String, String),
    MissingField(String),
    InvalidPath(String),
    ValidationError(String),
}

impl fmt::Display for ConfigErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigErrorKind::InvalidFormat(msg) => write!(f, "{}", msg),
            ConfigErrorKind::InvalidValue(field, msg) => write!(f, "{}: {}", field.bold(), msg),
            ConfigErrorKind::MissingField(field) => write!(f, "missing field: {}", field.bold()),
            ConfigErrorKind::InvalidPath(path) => write!(f, "invalid path: {}", path.bold()),
            ConfigErrorKind::ValidationError(msg) => write!(f, "{}", msg),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum LlaError {
    Io(io::Error),
    Parse(String),
    Config(ConfigErrorKind),
    Plugin(String),
    Filter(String),
    Other(String),
}

impl fmt::Display for LlaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LlaError::Io(err) => write!(f, "{}", err),
            LlaError::Parse(msg) => write!(f, "{}", msg),
            LlaError::Config(kind) => write!(f, "{}", kind),
            LlaError::Plugin(msg) => write!(f, "{}", msg),
            LlaError::Filter(msg) => write!(f, "{}", msg),
            LlaError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for LlaError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LlaError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for LlaError {
    fn from(err: io::Error) -> Self {
        LlaError::Io(err)
    }
}

impl From<toml::de::Error> for LlaError {
    fn from(err: toml::de::Error) -> Self {
        LlaError::Config(ConfigErrorKind::InvalidFormat(err.to_string()))
    }
}

impl From<dialoguer::Error> for LlaError {
    fn from(err: dialoguer::Error) -> Self {
        LlaError::Plugin(format!("interactive mode error: {}", err))
    }
}

impl From<serde_json::Error> for LlaError {
    fn from(err: serde_json::Error) -> Self {
        LlaError::Plugin(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, LlaError>;
