use super::{Command, CommandOpts, Execution};
use crate::{errors::RitError, repository::Repository, workspace::Entry};

pub struct Status {
    opts: CommandOpts,
    repo: Repository,
}

#[derive(Debug)]
pub struct StatusResult {
    pub untracked: Vec<Entry>,
}

impl Command for Status {
    fn new(opts: CommandOpts) -> Self {
        let repo = Repository::new(opts.dir.clone());

        Self { opts, repo }
    }

    fn execute(&mut self) -> Result<Execution, RitError> {
        let mut entries = self.repo.workspace.list_files(None);

        entries.sort_by_key(|entry| entry.path.clone());

        Ok(Execution::Status(StatusResult { untracked: entries }))
    }
}