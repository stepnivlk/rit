use crate::{database::Database, index::Index, refs::Refs, workspace::Workspace};
use std::path::PathBuf;

pub struct Repository {
    pub database: Database,
    pub index: Index,
    pub refs: Refs,
    pub workspace: Workspace,
}

impl Repository {
    pub fn new(project_path: PathBuf) -> Self {
        let git_path = project_path.join(".git");

        Self {
            database: Database::new(git_path.clone().join("objects")),
            index: Index::new(git_path.clone().join("index")),
            refs: Refs::new(git_path),
            workspace: Workspace::new(project_path),
        }
    }
}
