use crate::errors::RitError;
use std::path::PathBuf;

mod add;
mod commit;
mod init;
mod status;

pub use add::Add;
pub use commit::Commit;
pub use init::Init;
pub use status::Status;

#[derive(Clone)]
pub struct Session {
    pub author_name: String,
    pub author_email: String,
    pub project_dir: PathBuf,
}

pub trait Command {
    fn execute(&mut self) -> Result<Execution, RitError>;
}

#[derive(Debug)]
pub enum Execution {
    Empty,
    Commit(commit::CommitResult),
    Status(status::StatusResult),
}
