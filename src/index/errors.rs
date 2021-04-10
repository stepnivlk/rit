use crate::lockfile::LockError;
use std::{fmt, io};
#[derive(Debug)]
pub enum IndexError {
    Parse(String),
    Io,
    Lock(LockError),
}

impl fmt::Display for IndexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IndexError::Parse(msg) => write!(f, "{}", msg),
            IndexError::Io => write!(f, "Cannot access the index file"),
            IndexError::Lock(err) => write!(f, "{}", err),
        }
    }
}

impl From<io::Error> for IndexError {
    fn from(_err: io::Error) -> IndexError {
        IndexError::Io
    }
}
impl From<LockError> for IndexError {
    fn from(err: LockError) -> IndexError {
        IndexError::Lock(err)
    }
}
