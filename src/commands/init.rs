use super::{Command, CommandOpts, Execution};
use crate::errors::RitError;
use std::{fs, io::BufRead, path::PathBuf};

pub struct Init<R: BufRead>(CommandOpts<R>);

impl<R: BufRead> Init<R> {
    fn git_path(&mut self) -> PathBuf {
        self.0
            .args
            .get(0)
            .map(|path| PathBuf::from(path))
            .unwrap_or(self.0.dir.clone())
            .join(".git")
    }
}

impl<R: BufRead> Command<R> for Init<R> {
    fn new(opts: CommandOpts<R>) -> Self {
        Self(opts)
    }

    fn execute(&mut self) -> Result<Execution, RitError> {
        let git_path = self.git_path();

        for dir in &["objects", "refs"] {
            fs::create_dir_all(git_path.join(dir))?;
        }

        Ok(Execution::Empty)
    }
}
