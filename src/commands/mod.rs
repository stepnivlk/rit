use crate::errors::RitError;
use std::{io::BufRead, path::PathBuf};

mod init;
use init::Init;

mod commit;
use commit::Commit;

mod add;
use add::Add;

mod status;
use status::{Status, StatusResult};

#[derive(Clone, Debug)]
pub struct Session<R: BufRead> {
    pub name: String,
    pub email: String,
    input: R,
}

impl<R: BufRead> Session<R> {
    pub fn new(name: Option<String>, email: Option<String>, input: R) -> Result<Self, RitError> {
        let name = name.unwrap();
        let email = email.unwrap();

        Ok(Self { name, email, input })
    }

    pub fn read_input(&mut self) -> Result<String, RitError> {
        let mut text = String::new();

        self.input.read_to_string(&mut text)?;

        Ok(text)
    }
}

trait Command<R: BufRead> {
    fn new(opts: CommandOpts<R>) -> Self;

    fn execute(&mut self) -> Result<Execution, RitError>;
}

pub struct CommandOpts<R: BufRead> {
    pub dir: PathBuf,
    pub session: Session<R>,
    pub args: Vec<String>,
}

#[derive(Debug)]
pub enum Execution {
    Empty,
    Status(StatusResult),
}

pub fn execute<R: BufRead>(mut opts: CommandOpts<R>) -> Result<Execution, RitError> {
    let name = opts.args.remove(0);

    match &name[..] {
        "init" => Init::new(opts).execute(),
        "add" => Add::new(opts).execute(),
        "commit" => Commit::new(opts).execute(),
        "status" => Status::new(opts).execute(),
        _ => Err(RitError::UnknownCommand(name.to_string())),
    }
}
