use crate::errors::RitError;
use std::path::PathBuf;

mod init;
pub use init::Init;

mod commit;
pub use commit::Commit;

mod add;
pub use add::Add;

mod status;
pub use status::Status;
use status::StatusResult;

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
    Status(StatusResult),
}
