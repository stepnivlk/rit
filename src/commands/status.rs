use super::{Command, CommandOpts, Execution};
use crate::{errors::RitError, repository::Repository, workspace::Entry};
use std::io::BufRead;

pub struct Status<R: BufRead> {
    opts: CommandOpts<R>,
    repo: Repository,
}

#[derive(Debug)]
pub struct StatusResult {
    pub untracked: Vec<Entry>,
}

impl<R: BufRead> Command<R> for Status<R> {
    fn new(opts: CommandOpts<R>) -> Self {
        let repo = Repository::new(opts.dir.clone());

        Self { opts, repo }
    }

    fn execute(&mut self) -> Result<Execution, RitError> {
        self.repo.index.load()?;

        let mut entries = self.repo.workspace.list_files(None);

        entries.retain(|entry| !self.repo.index.is_tracked(&entry.relative_path_name));
        entries.sort_by_key(|entry| entry.relative_path.clone());

        Ok(Execution::Status(StatusResult { untracked: entries }))
    }
}
