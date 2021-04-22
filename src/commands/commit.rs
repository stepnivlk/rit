use super::{Command, CommandOpts, Execution};
use crate::{errors::RitError, id::Id, objects, repository::Repository};

pub struct Commit(CommandOpts);

impl Commit {
    fn commit<'a>(
        &'a self,
        parent_id: &'a Option<String>,
        root_id: Id,
        message: &'a str,
    ) -> objects::Commit {
        let author = objects::Author::new(&self.0.session.name, &self.0.session.email);

        objects::Commit::new(&parent_id, root_id, author, &message)
    }

    fn report(&self, parent_id: Option<String>, commit_id: Id, message: String) {
        let root_part = match parent_id {
            Some(_) => "",
            None => "(root-commit) ",
        };

        println!(
            "[{}{}] {}",
            root_part,
            commit_id.as_str,
            message.lines().next().unwrap()
        );
    }
}

impl Command for Commit {
    fn new(opts: CommandOpts) -> Self {
        Self(opts)
    }

    fn execute(&mut self) -> Result<Execution, RitError> {
        let mut repo = Repository::new(self.0.dir.clone());

        repo.index.load()?;

        let entries = repo.index.entries();

        let mut root = objects::Tree::build(entries);

        root.traverse(|tree| {
            let id = repo.database.store(tree).unwrap();
            tree.id = Some(id);
        });

        let root_id = repo.database.store(&mut root)?;
        let parent_id = repo.refs.read_head();

        let message = self.0.session.read_stdin()?;

        let mut commit = self.commit(&parent_id, root_id, &message);
        let commit_id = repo.database.store(&mut commit)?;
        repo.refs.update_head(&commit_id)?;

        self.report(parent_id, commit_id, message);

        Ok(Execution::Empty)
    }
}
