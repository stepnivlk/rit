use super::{Command, Execution};
use crate::{
    errors::RitError,
    index,
    objects::{Blob, Storable},
    repository::Repository,
    workspace, Session,
};
use std::{collections::HashMap, path::PathBuf};

pub struct Status {
    session: Session,
    repo: Repository,
    untracked: Vec<workspace::Entry>,
    modified: Vec<workspace::Entry>,
    deleted: Vec<workspace::Entry>,
    stats: HashMap<String, workspace::Stat>,
}

#[derive(Debug)]
pub struct StatusResult {
    pub untracked: Vec<workspace::Entry>,
    pub modified: Vec<workspace::Entry>,
    pub deleted: Vec<workspace::Entry>,
}

enum EntryChange {
    Changed,
    UpdateStat,
    Unchanged,
}

impl Status {
    pub fn new(session: Session) -> Self {
        let repo = Repository::new(session.project_dir.clone());

        Self {
            session,
            repo,
            untracked: vec![],
            modified: vec![],
            deleted: vec![],
            stats: HashMap::new(),
        }
    }

    fn detect_workspace_changes(&mut self) {
        for index_entry in self.repo.index.entries() {
            let workspace_entry = self.build_workspace_entry(&index_entry);

            match self.stats.get(&index_entry.pathname) {
                Some(stat) => {
                    match self.detect_entry_changes(&index_entry, &workspace_entry, stat) {
                        EntryChange::Changed => {
                            self.modified.push(workspace_entry);
                        }
                        EntryChange::UpdateStat => {
                            self.repo
                                .index
                                .update_entry_stat(&index_entry.pathname, stat);
                        }
                        _ => {}
                    };
                }
                None => self.deleted.push(workspace_entry),
            }
        }
    }

    fn detect_entry_changes(
        &self,
        index_entry: &index::Entry,
        workspace_entry: &workspace::Entry,
        stat: &workspace::Stat,
    ) -> EntryChange {
        if !index_entry.matches_stat(stat) {
            return EntryChange::Changed;
        }

        if index_entry.matches_times(stat) {
            return EntryChange::Unchanged;
        }

        let file = self.repo.workspace.read_file(&workspace_entry).unwrap();
        let mut blob = Blob::new(file);

        let id = blob.store(|(_, _)| {}).unwrap();

        if index_entry.id == id {
            return EntryChange::UpdateStat;
        }

        EntryChange::Changed
    }

    fn build_workspace_entry(&self, index_entry: &index::Entry) -> workspace::Entry {
        let absolute_path = self.session.project_dir.join(&index_entry.path);

        workspace::Entry::new(absolute_path, index_entry.path.clone())
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

    fn is_trackable_entry(&self, entry: &workspace::Entry) -> bool {
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
        self.repo.index.load_for_update()?;

        self.scan_workspace();
        self.detect_workspace_changes();

        self.repo.index.write_updates()?;

        // TODO: -clone
        Ok(Execution::Status(StatusResult {
            untracked: self.untracked.clone(),
            modified: self.modified.clone(),
            deleted: self.deleted.clone(),
        }))
    }
}
