use crate::{database::Database, index::Index, refs::Refs, workspace::Workspace};
use std::path::PathBuf;

pub struct Repository<'a> {
    pub database: Database,
    pub index: Index,
    pub refs: Refs<'a>,
    pub workspace: Workspace<'a>,
}

impl<'a> Repository<'a> {
    pub fn new(git_path: &'a PathBuf) -> Self {
        Self {
            database: Database::new(git_path.join("objects")),
            index: Index::new(git_path.join("index")),
            refs: Refs::new(&git_path),
            workspace: Workspace::new(&git_path),
        }
    }
}
