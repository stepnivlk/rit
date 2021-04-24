use super::{Command, Execution};
use crate::{errors::RitError, id::Id, objects, repository::Repository, Session};
use std::fmt;

pub struct Commit {
    session: Session,
    message: String,
    repo: Repository,
}

#[derive(Debug)]
pub struct CommitResult {
    parent_id: Option<String>,
    commit_id: String,
    message: String,
}

impl fmt::Display for CommitResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let root_part = match self.parent_id {
            Some(_) => "",
            None => "(root-commit) ",
        };

        write!(f, "[{}{}] {}", root_part, self.commit_id, self.message)
    }
}

impl Commit {
    pub fn new(session: Session, message: String) -> Self {
        let repo = Repository::new(session.project_dir.clone());

        Self {
            session,
            message,
            repo,
        }
    }

    fn commit<'a>(&'a self, parent_id: &'a Option<String>, root_id: Id) -> Result<Id, RitError> {
        let author = objects::Author::new(&self.session.author_name, &self.session.author_email);

        let mut commit = objects::Commit::new(&parent_id, root_id, author, &self.message);

        let commit_id = self.repo.database.store(&mut commit)?;

        Ok(commit_id)
    }

    fn get_root(&mut self) -> Result<objects::Tree, RitError> {
        self.repo.index.load()?;

        let entries = self.repo.index.entries();

        Ok(objects::Tree::build(entries))
    }

    fn get_result(&self, parent_id: Option<String>, commit_id: Id) -> CommitResult {
        CommitResult {
            parent_id,
            commit_id: commit_id.as_str,
            message: self.message.lines().next().unwrap().into(),
        }
    }
}

impl Command for Commit {
    fn execute(&mut self) -> Result<Execution, RitError> {
        let mut root = self.get_root()?;

        root.traverse(|tree| {
            let id = self.repo.database.store(tree).unwrap();

            tree.id = Some(id);
        });

        let root_id = self.repo.database.store(&mut root)?;
        let parent_id = self.repo.refs.read_head();
        let commit_id = self.commit(&parent_id, root_id)?;

        self.repo.refs.update_head(&commit_id)?;

        Ok(Execution::Commit(self.get_result(parent_id, commit_id)))
    }
}
