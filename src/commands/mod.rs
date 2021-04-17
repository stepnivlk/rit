use crate::errors::RitError;
use std::{
    io::{self, Read},
    path::PathBuf,
};

mod init;
use init::Init;

mod commit;
use commit::Commit;

mod add;
use add::Add;

#[derive(Clone, Debug)]
pub struct Session {
    pub name: String,
    pub email: String,
}

impl Session {
    pub fn new(name: Option<String>, email: Option<String>) -> Result<Self, RitError> {
        let name = name.unwrap();
        let email = email.unwrap();

        Ok(Self { name, email })
    }

    pub fn read_stdin(&self) -> Result<String, RitError> {
        let mut text = String::new();

        io::stdin().read_to_string(&mut text)?;

        Ok(text)
    }
}

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
