use crate::{index::IndexError, lockfile::LockError, refs::RefsError};
use std::{env, fmt, io};

#[derive(Debug)]
pub enum RitError {
    Io(io::Error),
    Env,
    Index(IndexError),
    Lock(LockError),
    Refs(RefsError),
    MissingFile(String),
    PermissionDenied(String),
    UnknownCommand(String),
}

impl fmt::Display for RitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            RitError::Io(err) => write!(f, "IO failed: {:?}", err),
            RitError::Env => write!(f, "ENV access failed"),
            RitError::MissingFile(pathname) => {
                write!(f, "pathspec '{}' did not match any files", pathname)
            }
            RitError::PermissionDenied(pathname) => {
                write!(f, "open('{}'): Permission denied", pathname)
            }
            err => write!(f, "Internal error: {:?}", err),
        }
    }
}

impl From<io::Error> for RitError {
    fn from(err: io::Error) -> RitError {
        RitError::Io(err)
    }
}

impl From<env::VarError> for RitError {
    fn from(_err: env::VarError) -> RitError {
        RitError::Env
    }
}

impl From<RefsError> for RitError {
    fn from(err: RefsError) -> RitError {
        RitError::Refs(err)
    }
}

impl From<LockError> for RitError {
    fn from(err: LockError) -> RitError {
        RitError::Lock(err)
    }
}

impl From<IndexError> for RitError {
    fn from(err: IndexError) -> RitError {
        RitError::Index(err)
    }
}
