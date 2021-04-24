use super::{Command, Execution};
use crate::{errors::RitError, id::Id, objects, repository::Repository, Session};

pub struct Commit {
    session: Session,
    message: String,
}

impl Commit {
    pub fn new(session: Session, message: String) -> Self {
        Self { session, message }
    }

    fn commit<'a>(
        &'a self,
        parent_id: &'a Option<String>,
        root_id: Id,
        message: &'a str,
    ) -> objects::Commit {
        let author = objects::Author::new(&self.session.author_name, &self.session.author_email);

        objects::Commit::new(&parent_id, root_id, author, &message)
    }

    fn report(&self, parent_id: Option<String>, commit_id: Id) {
        let root_part = match parent_id {
            Some(_) => "",
            None => "(root-commit) ",
        };

        // TODO: Move to bin
        println!(
            "[{}{}] {}",
            root_part,
            commit_id.as_str,
            self.message.lines().next().unwrap()
        );
    }
}

impl Command for Commit {
    fn execute(&mut self) -> Result<Execution, RitError> {
        let mut repo = Repository::new(self.session.project_dir.clone());

        repo.index.load()?;

        let entries = repo.index.entries();

        let mut root = objects::Tree::build(entries);

        root.traverse(|tree| {
            let id = repo.database.store(tree).unwrap();
            tree.id = Some(id);
        });

        let root_id = repo.database.store(&mut root)?;
        let parent_id = repo.refs.read_head();

        let mut commit = self.commit(&parent_id, root_id, &self.message);
        let commit_id = repo.database.store(&mut commit)?;
        repo.refs.update_head(&commit_id)?;

        self.report(parent_id, commit_id);

        Ok(Execution::Empty)
    }
}
