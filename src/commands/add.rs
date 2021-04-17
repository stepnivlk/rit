use super::{Command, CommandOpts};
use crate::{errors::RitError, objects, repository::Repository};
use std::path::PathBuf;

pub struct Add {
    opts: CommandOpts,
    repo: Repository,
}

impl Add {
    fn expanded_paths(&mut self) -> Result<Vec<PathBuf>, RitError> {
        let mut files: Vec<PathBuf> = vec![];

        // TODO: -clone
        for path in self.opts.args.clone() {
            let path = self.repo.workspace.expand_path(&path);
            let path = path.map_err(|err| {
                self.repo.index.release_lock().unwrap();

                err
            })?;

            for file in self.repo.workspace.list_files(Some(&path)) {
                files.push(file);
            }
        }

        Ok(files)
    }

    fn add_to_index(&mut self, path: PathBuf) -> Result<(), RitError> {
        let data = self.repo.workspace.read_file(&path).map_err(|err| {
            self.repo.index.release_lock().unwrap();

            err
        })?;
        let stat = self.repo.workspace.stat_file(&data);

        let mut blob = objects::Blob::new(data);

        let blob_id = self.repo.database.store(&mut blob).unwrap();

        self.repo.index.add(path, blob_id, stat);

        Ok(())
    }
}

impl Command for Add {
    fn new(opts: CommandOpts) -> Self {
        let repo = Repository::new(opts.dir.clone());

        Self { opts, repo }
    }

    fn execute(&mut self) -> Result<(), RitError> {
        self.repo.index.load_for_update()?;

        let expanded_paths = self.expanded_paths()?;

        for path in expanded_paths {
            self.add_to_index(path)?;
        }

        self.repo.index.write_updates()?;

        Ok(())
    }
}
