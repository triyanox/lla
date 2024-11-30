use std::fmt;
use std::io;

#[derive(Debug)]
pub enum LlaError {
    Io(io::Error),
    Parse(String),
    Config(String),
    Plugin(String),
}

impl fmt::Display for LlaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LlaError::Io(err) => write!(f, "I/O error: {}", err),
            LlaError::Parse(msg) => write!(f, "Parse error: {}", msg),
            LlaError::Config(msg) => write!(f, "Configuration error: {}", msg),
            LlaError::Plugin(msg) => write!(f, "Plugin error: {}", msg),
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
        LlaError::Parse(err.to_string())
    }
}

impl From<dialoguer::Error> for LlaError {
    fn from(err: dialoguer::Error) -> Self {
        LlaError::Plugin(format!("Interactive mode error: {}", err))
    }
}

pub type Result<T> = std::result::Result<T, LlaError>;
