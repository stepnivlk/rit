use std::env;

fn exit(result: Result<rit::Execution, rit::errors::RitError>) {
    std::process::exit(match result {
        Ok(res) => {
            match res {
                rit::Execution::Status(res) => {
                    for untracked in res.untracked.iter() {
                        println!("?? {}", untracked.pathname);
                    }

                    0
                }
                _ => 0
            }
        },
        Err(err) => match err {
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

    let name = env::var("GIT_AUTHOR_NAME").ok();
    let email = env::var("GIT_AUTHOR_EMAIL").ok();
    let session = rit::Session::new(name, email).unwrap();

    let result = rit::execute(rit::CommandOpts { dir, session, args });

    exit(result);
}
