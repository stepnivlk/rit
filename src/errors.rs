use crate::refs::RefsError;
use std::{env, fmt, io};

#[derive(Debug)]
pub enum RitError {
    Io,
    Env,
    Internal,
}

impl fmt::Display for RitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RitError::Io => write!(f, "io failed"),
            RitError::Env => write!(f, "cannot read env"),
            RitError::Internal => write!(f, "Internal error"),
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
    fn from(_err: RefsError) -> RitError {
        RitError::Internal
    }
}
