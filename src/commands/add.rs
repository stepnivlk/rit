use super::{Command, Execution};
use crate::{errors::RitError, objects, repository::Repository, workspace::Entry, Session};

pub struct Add {
    paths: Vec<String>,
    repo: Repository,
}

impl Add {
    pub fn new(session: Session, paths: Vec<String>) -> Self {
        let repo = Repository::new(session.project_dir);

        Self { paths, repo }
    }

    fn expanded_entries(&mut self) -> Result<Vec<Entry>, RitError> {
        let mut entries: Vec<Entry> = vec![];

        // TODO: -clone
        for path in self.paths.clone() {
            let path = self.repo.workspace.expand_path(&path).map_err(|err| {
                self.repo.index.release_lock().unwrap();

                err
            })?;

            for entry in self.repo.workspace.list_files(Some(&path)) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    fn add_to_index(&mut self, entry: Entry) -> Result<(), RitError> {
        let file = self.repo.workspace.read_file(&entry).map_err(|err| {
            self.repo.index.release_lock().unwrap();

            err
        })?;
        let stat = self.repo.workspace.stat_file(&file);

        let mut blob = objects::Blob::new(file);

        let blob_id = self.repo.database.store(&mut blob).unwrap();

        self.repo.index.add(entry, blob_id, stat);

        Ok(())
    }
}

impl Command for Add {
    fn execute(&mut self) -> Result<Execution, RitError> {
        self.repo.index.load_for_update()?;

        let expanded_entries = self.expanded_entries()?;

        for entry in expanded_entries {
            self.add_to_index(entry)?;
        }

        self.repo.index.write_updates()?;

        Ok(Execution::Empty)
    }
}
