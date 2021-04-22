use super::{Command, CommandOpts, Execution};
use crate::{errors::RitError, objects, repository::Repository, workspace::Entry};
use std::io::BufRead;

pub struct Add<R: BufRead> {
    opts: CommandOpts<R>,
    repo: Repository,
}

impl<R: BufRead> Add<R> {
    fn expanded_entries(&mut self) -> Result<Vec<Entry>, RitError> {
        let mut entries: Vec<Entry> = vec![];

        // TODO: -clone
        for path in self.opts.args.clone() {
            let path = self.repo.workspace.expand_path(&path);
            let path = path.map_err(|err| {
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

impl<R: BufRead> Command<R> for Add<R> {
    fn new(opts: CommandOpts<R>) -> Self {
        let repo = Repository::new(opts.dir.clone());

        Self { opts, repo }
    }

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
