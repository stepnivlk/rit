use commands::CommandOpts;
use io::Read;
use lockfile::LockError;
use std::{env, io};

mod objects;

mod workspace;

mod database;

mod errors;
use errors::RitError;

mod refs;

mod lockfile;

mod index;
use index::IndexError;

mod id;

mod repository;

mod commands;

pub struct Session {
    pub name: String,
    pub email: String,
}

impl Session {
    fn new() -> Result<Self, RitError> {
        let name = env::var("GIT_AUTHOR_NAME")?;
        let email = env::var("GIT_AUTHOR_EMAIL")?;

        Ok(Self { name, email })
    }

    pub fn read_stdin(&self) -> Result<String, RitError> {
        let mut text = String::new();

        io::stdin().read_to_string(&mut text)?;

        Ok(text)
    }
}

fn exit(result: Result<(), RitError>) {
    std::process::exit(match result {
        Ok(_) => 0,
        Err(err) => match err {
            RitError::MissingFile(_) => {
                eprintln!("fatal: {}", err);
                128
            }
            RitError::PermissionDenied(_) => {
                eprintln!("error: {}", err);
                eprintln!("fatal: adding files failed");
                128
            }
            RitError::Lock(err) => match err {
                LockError::Denied(_) => {
                    eprintln!("fatal: {}", err);
                    128
                }
                _ => {
                    eprintln!("fatal: {:?}", err);
                    1
                }
            },
            RitError::Index(err) => match err {
                IndexError::Lock(lock_err @ LockError::Denied(_)) => {
                    eprintln!(
                        "fatal: {}
                        
Another rit process seems to be running in this repository.
Please make sure all processes are terminated then try again.
If it still fails, a jit process may have crashed in this
repository earlier: remove the file manually to continue.",
                        lock_err
                    );
                    128
                }
                _ => {
                    eprintln!("fatal: {:?}", err);
                    1
                }
            },
            _ => {
                eprintln!("fatal: {:?}", err);
                1
            }
        },
    });
}

fn main() {
    let mut args = env::args();
    args.next();

    let args = args.collect();
    let dir = env::current_dir().unwrap();
    let session = Session::new().unwrap();

    let result = commands::execute(CommandOpts { dir, session, args });

    exit(result);
}
