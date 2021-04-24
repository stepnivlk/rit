use super::{Command, Execution};
use crate::{
    errors::RitError,
    repository::Repository,
    workspace::{self, Entry, Stat},
    Session,
};
use std::{collections::HashMap, path::PathBuf};

pub struct Status {
    session: Session,
    repo: Repository,
    untracked: Vec<Entry>,
    changed: Vec<Entry>,
    stats: HashMap<String, Stat>,
}

#[derive(Debug)]
pub struct StatusResult {
    pub untracked: Vec<Entry>,
    pub changed: Vec<Entry>,
}

impl Status {
    pub fn new(session: Session) -> Self {
        let repo = Repository::new(session.project_dir.clone());

        Self {
            session,
            repo,
            untracked: vec![],
            changed: vec![],
            stats: HashMap::new(),
        }
    }

    fn detect_workspace_changes(&mut self) {
        for index_entry in self.repo.index.entries() {
            if let Some(stat) = self.stats.get(&index_entry.pathname) {
                if !index_entry.matches_stat(stat) {
                    let absolute_path = self.session.project_dir.join(&index_entry.path);
                    let workspace_entry = workspace::Entry::new(absolute_path, index_entry.path);

                    self.changed.push(workspace_entry);
                }
            }
        }
    }

    fn scan_workspace(&mut self) {
        self.do_scan_workspace(None);

        self.untracked
            .sort_by_key(|entry| entry.relative_path.clone());
    }

    fn do_scan_workspace(&mut self, prefix: Option<&PathBuf>) {
        for (entry, stat) in self.repo.workspace.list_dir(prefix) {
            if self.repo.index.is_tracked(&entry.relative_path_name) {
                if entry.is_dir {
                    self.do_scan_workspace(Some(&entry.absolute_path));
                } else {
                    self.stats.insert(entry.relative_path_name, stat);
                }
            } else if self.is_trackable_entry(&entry) {
                self.untracked.push(entry);
            }
        }
    }

    fn is_trackable_entry(&self, entry: &Entry) -> bool {
        if !entry.is_dir {
            return !self.repo.index.is_tracked(&entry.relative_path_name);
        }

        let mut nested_entries = self.repo.workspace.list_dir(Some(&entry.absolute_path));

        nested_entries.sort_by_key(|(entry, _)| entry.is_dir);

        nested_entries
            .iter()
            .any(|(entry, _)| self.is_trackable_entry(entry))
    }
}

impl Command for Status {
    fn execute(&mut self) -> Result<Execution, RitError> {
        self.repo.index.load()?;

        self.scan_workspace();
        self.detect_workspace_changes();

        // TODO: -clone
        Ok(Execution::Status(StatusResult {
            untracked: self.untracked.clone(),
            changed: self.changed.clone(),
        }))
    }
}
