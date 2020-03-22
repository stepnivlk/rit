use std::{env, error, fmt, io};

#[derive(Debug)]
pub enum RitError {
    Io,
    Env,
}

impl fmt::Display for RitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RitError::Io => write!(f, "io failed"),
            RitError::Env => write!(f, "cannot read env"),
        }
    }
}

impl error::Error for RitError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
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
