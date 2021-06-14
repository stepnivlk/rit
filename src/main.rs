use std::env;

use rit::Command;

fn handle_err(err: rit::errors::RitError) -> i32 {
    match err {
        rit::errors::RitError::MissingFile(_) => {
            eprintln!("fatal: {}", err);
            128
        }
        rit::errors::RitError::PermissionDenied(_) => {
            eprintln!("error: {}", err);
            eprintln!("fatal: adding files failed");
            128
        }
        rit::errors::RitError::Lock(err) => match err {
            rit::lockfile::LockError::Denied(_) => {
                eprintln!("fatal: {}", err);
                128
            }
            _ => {
                eprintln!("fatal: {:?}", err);
                1
            }
        },
        rit::errors::RitError::Index(err) => match err {
            rit::index::IndexError::Lock(lock_err @ rit::lockfile::LockError::Denied(_)) => {
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
        rit::errors::RitError::UnknownCommand(command) => {
            eprintln!("rit: '{}' is not a rit command. See 'rit --help'.", command);
            1
        }
        _ => {
            eprintln!("fatal: {:?}", err);
            1
        }
    }
}

fn handle_ok(execution: rit::Execution) -> i32 {
    match execution {
        rit::Execution::Status(res) => {
            for untracked in res.untracked {
                println!("?? {}", untracked);
            }

            for modified in res.modified {
                println!(" M {}", modified);
            }

            for deleted in res.deleted {
                println!(" D {}", deleted);
            }

            0
        }
        rit::Execution::Commit(res) => {
            println!("{}", res);

            0
        }
        _ => 0,
    }
}

fn get_session() -> rit::Session {
    let project_dir = env::current_dir().unwrap();
    let author_name = env::var("GIT_AUTHOR_NAME").unwrap();
    let author_email = env::var("GIT_AUTHOR_EMAIL").unwrap();

    rit::Session {
        author_name,
        author_email,
        project_dir,
    }
}

fn main() {
    let mut args = env::args();
    args.next();

    let session = get_session();

    let result = match args.next().as_deref() {
        Some("init") => {
            let path = args.next();

            rit::Init::new(session, path).execute()
        }
        Some("add") => {
            let paths = args.collect();

            rit::Add::new(session, paths).execute()
        }
        Some("commit") => {
            let message = args.next().unwrap();

            rit::Commit::new(session, message).execute()
        }
        Some("status") => rit::Status::new(session).execute(),
        Some(name) => {
            let err = rit::errors::RitError::UnknownCommand(name.to_string());

            Err(err)
        }
        _ => {
            println!("TODO: Help");

            Ok(rit::Execution::Empty)
        }
    }
    .map(handle_ok)
    .map_err(handle_err);

    std::process::exit(match result {
        Ok(code) => code,
        Err(code) => code,
    })
}
