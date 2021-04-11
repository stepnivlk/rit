use crate::{errors::RitError, Session};
use std::path::PathBuf;

mod init;
use init::Init;

mod commit;
use commit::Commit;

mod add;
use add::Add;

trait Command {
    fn execute(&mut self) -> Result<(), RitError>;
}

pub struct CommandOpts {
    pub dir: PathBuf,
    pub session: Session,
    pub args: Vec<String>,
}

pub fn execute(mut opts: CommandOpts) -> Result<(), RitError> {
    let name = opts.args.remove(0);

    match &name[..] {
        "init" => Init(opts).execute(),
        "add" => Add(opts).execute(),
        "commit" => Commit(opts).execute(),
        _ => Err(RitError::UnknownCommand),
    }
}
