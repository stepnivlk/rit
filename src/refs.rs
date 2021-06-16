use crate::{
    id::Id,
    lockfile::{LockError, Lockfile},
};
use std::{fmt, fs, io::{self, BufReader, prelude::*}, path::PathBuf};

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

pub struct Refs(PathBuf);

impl Refs {
    pub fn new(path: PathBuf) -> Self {
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

    // TODO:
    pub fn read_head(&self) -> Option<String> {
        let file = fs::File::open(self.head_path()).unwrap();

        let reader = BufReader::new(file);

        reader.lines().next().unwrap().ok()
    }

    fn head_path(&self) -> PathBuf {
        self.0.join("HEAD")
    }
}
