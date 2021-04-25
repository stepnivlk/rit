use super::{Command, Execution};
use crate::{errors::RitError, Session};
use std::{fs, path::PathBuf};

pub struct Init {
    session: Session,
    path: Option<String>,
}

impl Init {
    pub fn new(session: Session, path: Option<String>) -> Self {
        Self { session, path }
    }

    fn git_path(&mut self) -> PathBuf {
        let relative_path = self
            .path
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| self.session.project_dir.clone())
            .join(".git");

        self.session.project_dir.join(relative_path)
    }
}

impl Command for Init {
    fn execute(&mut self) -> Result<Execution, RitError> {
        let git_path = self.git_path();
        dbg!(&git_path);

        for dir in &["objects", "refs"] {
            fs::create_dir_all(git_path.join(dir))?;
        }

        Ok(Execution::Empty)
    }
}
