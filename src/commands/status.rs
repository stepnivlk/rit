use super::{Command, CommandOpts, Execution};
use crate::{errors::RitError, repository::Repository, workspace::Entry};
use std::io::BufRead;
use std::path::PathBuf;

pub struct Status<R: BufRead> {
    opts: CommandOpts<R>,
    repo: Repository,
    untracked: Vec<Entry>,
}

impl<R: BufRead> Status<R> {
    fn scan_workspace(&mut self, prefix: Option<&PathBuf>) {
        for entry in self.repo.workspace.list_dir(prefix) {
            if self.repo.index.is_tracked(&entry.relative_path_name) {
                if entry.is_dir {
                    self.scan_workspace(Some(&entry.absolute_path));
                }
            } else {
                self.untracked.push(entry);
            }
        }
    }
}

#[derive(Debug)]
pub struct StatusResult {
    pub untracked: Vec<Entry>,
}

impl<R: BufRead> Command<R> for Status<R> {
    fn new(opts: CommandOpts<R>) -> Self {
        let repo = Repository::new(opts.dir.clone());

        Self {
            opts,
            repo,
            untracked: vec![],
        }
    }

    fn execute(&mut self) -> Result<Execution, RitError> {
        self.repo.index.load()?;

        self.scan_workspace(None);

        self.untracked
            .sort_by_key(|entry| entry.relative_path.clone());

        // TODO: -clone
        Ok(Execution::Status(StatusResult {
            untracked: self.untracked.clone(),
        }))
    }
}
