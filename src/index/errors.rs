use crate::lockfile::LockError;
use std::{error::Error, fmt, io};
#[derive(Debug)]
pub enum IndexError {
    Parse(String),
    Io,
    Lock,
}

impl fmt::Display for IndexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IndexError::Parse(msg) => write!(f, "{}", msg),
            IndexError::Io => write!(f, "Cannot access the index file"),
            IndexError::Lock => write!(f, "Cannot acquire the index lockfile"),
        }
    }
}

impl Error for IndexError {
    fn description(&self) -> &str {
        match &self {
            IndexError::Parse(msg) => &msg,
            IndexError::Io => "Cannot access the index file",
            IndexError::Lock => "Cannot acquire the index lockfile",
        }
    }
}

impl From<io::Error> for IndexError {
    fn from(_err: io::Error) -> IndexError {
        IndexError::Io
    }
}

impl From<LockError> for IndexError {
    fn from(_err: LockError) -> IndexError {
        IndexError::Lock
    }
}
