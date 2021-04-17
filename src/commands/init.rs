use super::{Command, CommandOpts};
use crate::errors::RitError;
use std::{fs, path::PathBuf};

pub struct Init(pub CommandOpts);

impl Init {
    fn git_path(&mut self) -> PathBuf {
        self.0
            .args
            .get(0)
            .map(|path| PathBuf::from(path))
            .unwrap_or(self.0.dir.clone())
            .join(".git")
    }
}

impl Command for Init {
    fn execute(&mut self) -> Result<(), RitError> {
        let git_path = self.git_path();
        dbg!(&git_path);

        for dir in &["objects", "refs"] {
            fs::create_dir_all(git_path.join(dir))?;
        }

        Ok(())
    }
}
