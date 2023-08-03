use std::error::Error;

pub enum LlaError {
    FailedToReadDir(String),
    FailedToGetMetadata(String),
    FailedToGetUserByUID(String),
    FailedToGetGroupByGID(String),
}

impl std::fmt::Debug for LlaError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LlaError::FailedToReadDir(dir) => write!(f, "Failed to read directory: {}", dir),
            LlaError::FailedToGetMetadata(file) => {
                write!(f, "Failed to get metadata for file: {}", file)
            }
            LlaError::FailedToGetUserByUID(uid) => write!(f, "Failed to get user by UID: {}", uid),
            LlaError::FailedToGetGroupByGID(gid) => {
                write!(f, "Failed to get group by GID: {}", gid)
            }
        }
    }
}

impl std::fmt::Display for LlaError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LlaError::FailedToReadDir(dir) => write!(f, "Failed to read directory: {}", dir),
            LlaError::FailedToGetMetadata(file) => {
                write!(f, "Failed to get metadata for file: {}", file)
            }
            LlaError::FailedToGetUserByUID(uid) => write!(f, "Failed to get user by UID: {}", uid),
            LlaError::FailedToGetGroupByGID(gid) => {
                write!(f, "Failed to get group by GID: {}", gid)
            }
        }
    }
}

impl Error for LlaError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            LlaError::FailedToReadDir(_) => None,
            LlaError::FailedToGetMetadata(_) => None,
            LlaError::FailedToGetUserByUID(_) => None,
            LlaError::FailedToGetGroupByGID(_) => None,
        }
    }
}
