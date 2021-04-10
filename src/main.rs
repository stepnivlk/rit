use io::Read;
use lockfile::LockError;
use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

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
use repository::Repository;

fn handle_init(path: &Path) -> Result<(), RitError> {
    let path = path.join(".git");

    for dir in &["objects", "refs"] {
        fs::create_dir_all(path.join(dir))?;
    }

    Ok(())
}

fn env_data() -> Result<(String, String, String), RitError> {
    let name = env::var("GIT_AUTHOR_NAME")?;
    let email = env::var("GIT_AUTHOR_EMAIL")?;
    let mut message = String::new();
    io::stdin().read_to_string(&mut message)?;

    Ok((name, email, message))
}

fn handle_commit(path: &Path) -> Result<(), RitError> {
    let git_path = path.join(".git");
    let mut repo = Repository::new(&git_path);

    repo.index.load()?;

    let entries = repo.index.entries();

    let mut root = objects::Tree::build(entries);

    root.traverse(|tree| {
        let id = repo.database.store(tree).unwrap();
        tree.id = Some(id);
    });

    let root_id = repo.database.store(&mut root)?;

    let parent_id = repo.refs.read_head();
    let (name, email, message) = env_data()?;

    let author = objects::Author::new(name, email);

    let mut commit = objects::Commit::new(&parent_id, root_id, author, &message);
    let commit_id = repo.database.store(&mut commit)?;
    repo.refs.update_head(&commit_id)?;

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

    Ok(())
}

fn handle_add(paths: Vec<&String>) -> Result<(), RitError> {
    let root_path = env::current_dir()?;
    let git_path = root_path.join(".git");
    let mut repo = Repository::new(&git_path);

    repo.index.load_for_update()?;

    let mut files: Vec<PathBuf> = vec![];

    for path in paths {
        let path = repo.workspace.expand_path(path);
        let path = path.map_err(|err| {
            repo.index.release_lock().unwrap();

            err
        })?;

        for file in repo.workspace.list_files(Some(&path)) {
            files.push(file);
        }
    }

    for file in files {
        let data = repo.workspace.read_file(&file).map_err(|err| {
            repo.index.release_lock().unwrap();

            err
        })?;
        let stat = repo.workspace.stat_file(&data);

        let mut blob = objects::Blob::new(data);

        let blob_id = repo.database.store(&mut blob).unwrap();

        repo.index.add(file, blob_id, stat);
    }

    repo.index.write_updates()?;

    Ok(())
}

fn handle_result(result: Result<(), RitError>) {
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

fn main() -> Result<(), RitError> {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => eprintln!("Please provide command"),
        2 => {
            let cmd = &args[1];

            let root_path = env::current_dir()?;

            match &cmd[..] {
                "init" => {
                    handle_init(&root_path)?;
                }
                "commit" => {
                    handle_commit(&root_path)?;
                }
                c => eprintln!("Command {} not supported", c),
            }
        }
        3 => {
            let cmd = &args[1];
            match &cmd[..] {
                "init" => {
                    let path = Path::new(&args[2]);
                    handle_init(&path)?;
                }
                "add" => {
                    let paths = args[2..].iter().collect::<Vec<&String>>();

                    let result = handle_add(paths);

                    handle_result(result);
                }

                c => eprintln!("Command {} not supported", c),
            }
        }
        _ => {
            let cmd = &args[1];
            match &cmd[..] {
                "add" => {
                    let paths = args[2..].iter().collect::<Vec<&String>>();

                    let result = handle_add(paths);

                    handle_result(result);
                }

                c => eprintln!("Command {} not supported", c),
            }
        }
    }

    Ok(())
}
