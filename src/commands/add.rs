use super::{Command, CommandOpts};
use crate::{errors::RitError, objects, repository::Repository};
use std::path::PathBuf;

pub struct Add(pub CommandOpts);

impl Command for Add {
    fn execute(&mut self) -> Result<(), RitError> {
        let mut repo = Repository::new(self.0.dir.clone());

        repo.index.load_for_update()?;

        let mut files: Vec<PathBuf> = vec![];

        // TODO: -clone
        for path in self.0.args.clone() {
            let path = repo.workspace.expand_path(&path);
            let path = path.map_err(|err| {
                repo.index.release_lock().unwrap();

                err
            })?;

            for file in repo.workspace.list_files(Some(&path)) {
                files.push(file);
            }
        }

        for file in files {
            let data = repo.workspace.read_file(&file).map_err(|err| {
                repo.index.release_lock().unwrap();

                err
            })?;
            let stat = repo.workspace.stat_file(&data);

            let mut blob = objects::Blob::new(data);

            let blob_id = repo.database.store(&mut blob).unwrap();

            repo.index.add(file, blob_id, stat);
        }

        repo.index.write_updates()?;

        Ok(())
    }
}
