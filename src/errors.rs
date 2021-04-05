use crate::{index::IndexError, lockfile::LockError, refs::RefsError};
use std::{env, error::Error, fmt, io};

#[derive(Debug)]
pub enum RitError {
    Io,
    Env,
    Index(IndexError),
    Lock(LockError),
    Refs(RefsError),
}

impl Error for RitError {
    fn description(&self) -> &str {
        match &self {
            RitError::Io => "IO failed",
            RitError::Env => "ENV access failed",
            _ => "Internal error",
        }
    }
}

impl fmt::Display for RitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RitError::Io => write!(f, "IO failed"),
            RitError::Env => write!(f, "ENV access failed"),
            err => write!(f, "Internal error: {}", err),
        }
    }
}

impl From<io::Error> for RitError {
    fn from(_err: io::Error) -> RitError {
        RitError::Io
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
