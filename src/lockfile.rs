use std::{
    fmt,
    fs::{self, File, OpenOptions},
    io::{self, prelude::*, ErrorKind},
    path::PathBuf,
};

#[derive(Debug)]
pub enum LockError {
    StaleLock,
    Denied,
    Other(ErrorKind),
}

impl fmt::Display for LockError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LockError::StaleLock => write!(f, "Not holding lock"),
            LockError::Denied => write!(f, "Could not acquire lock"),
            LockError::Other(_) => write!(f, "Something went wrong..."),
        }
    }
}

impl From<io::Error> for LockError {
    fn from(err: io::Error) -> LockError {
        LockError::Other(err.kind())
    }
}

pub struct Lockfile {
    pub file_path: PathBuf,
    lock_path: PathBuf,
    lock: Option<File>,
}

impl Lockfile {
    pub fn new(mut path: PathBuf) -> Self {
        let file_path = path.clone();
        path.set_extension("lock");

        Self {
            file_path,
            lock_path: path,
            lock: None,
        }
    }

    pub fn hold_for_update(&mut self) -> Result<(), LockError> {
        OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&self.lock_path)
            .map(|f| self.lock = Some(f))
            .map_or(Err(LockError::Denied), |_| Ok(()))
    }

    pub fn write(&mut self, content: &[u8]) -> Result<(), LockError> {
        self.guard_stale_lock()?;

        if let Some(l) = self.lock.as_mut() {
            l.write_all(content)?;
        }

        Ok(())
    }

    pub fn commit(&mut self) -> Result<(), LockError> {
        self.guard_stale_lock()?;

        fs::rename(&self.lock_path, &self.file_path)?;
        self.lock = None;

        Ok(())
    }

    pub fn rollback(&mut self) -> Result<(), LockError> {
        self.guard_stale_lock()?;

        fs::remove_file(self.lock_path.to_str().unwrap())?;
        self.lock = None;

        Ok(())
    }

    fn guard_stale_lock(&self) -> Result<(), LockError> {
        match self.lock {
            Some(_) => Ok(()),
            None => Err(LockError::StaleLock),
        }
    }
}
