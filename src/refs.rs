use crate::{
    id::Id,
    lockfile::{LockError, Lockfile},
};
use std::{
    fmt, fs,
    io::{self, prelude::*},
    path::PathBuf,
};

#[derive(Debug)]
pub struct RefsError;

impl fmt::Display for RefsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RefsError")
    }
}

impl From<io::Error> for RefsError {
    fn from(_err: io::Error) -> RefsError {
        RefsError
    }
}

impl From<LockError> for RefsError {
    fn from(_err: LockError) -> RefsError {
        RefsError
    }
}

pub struct Refs<'a>(&'a PathBuf);

impl<'a> Refs<'a> {
    pub fn new(path: &'a PathBuf) -> Self {
        Self(path)
    }

    pub fn update_head(&self, id: &Id) -> Result<(), RefsError> {
        let mut lockfile = Lockfile::new(self.head_path());
        lockfile.hold_for_update()?;

        lockfile.write(&id.as_str.as_bytes())?;
        lockfile.write("\n".as_bytes())?;
        lockfile.commit()?;

        Ok(())
    }

    pub fn read_head(&self) -> Option<String> {
        let file = fs::File::open(self.head_path()).ok();
        let mut head = String::new();

        file.and_then(|mut f| f.read_to_string(&mut head).ok())
            .map(|_| head)
    }

    fn head_path(&self) -> PathBuf {
        self.0.join("HEAD")
    }
}
