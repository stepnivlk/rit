use io::prelude::*;
use io::Read;
use std::{env, fs, io, path::Path};

mod objects;

mod workspace;
use workspace::Workspace;

mod database;
use database::Database;

mod errors;
use errors::RitError;

fn handle_init(path: &Path) -> Result<(), RitError> {
    let path = path.join(".git");

    for dir in &["objects", "refs"] {
        fs::create_dir_all(path.join(dir))?;
    }

    Ok(())
}

fn get_env_data() -> Result<(String, String, String), RitError> {
    let name = env::var("GIT_AUTHOR_NAME")?;
    let email = env::var("GIT_AUTHOR_EMAIL")?;
    let mut message = String::new();
    io::stdin().read_to_string(&mut message)?;

    Ok((name, email, message))
}

fn handle_commit(path: &Path) -> Result<(), RitError> {
    let git_path = path.join(".git");
    let db_path = git_path.join("objects");

    let workspace = Workspace::new(&path);
    let database = Database::new(&db_path);

    let entries: Vec<objects::Entry> = workspace
        .list_files()?
        .map(|entry| {
            let file = workspace.read_file(&entry).unwrap();
            let blob = objects::Blob::new(file);

            let blob_id = database.store(blob).unwrap();

            objects::Entry {
                id: blob_id,
                path: entry,
            }
        })
        .collect();

    let tree = objects::Tree::new(entries);
    let tree_id = database.store(tree)?;

    let (name, email, message) = get_env_data()?;

    let author = objects::Author::new(name, email);

    let commit = objects::Commit::new(tree_id, author, &message);
    let commit_id = database.store(commit)?;

    let mut file = fs::File::create(git_path.join("HEAD"))?;
    file.write_all(&commit_id.as_bytes)?;

    println!(
        "[(root-commit) {}] {}",
        commit_id.as_str,
        message.lines().next().unwrap()
    );

    Ok(())
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
                c => eprintln!("Command {} not supported", c),
            }
        }
        _ => eprintln!("Non-valid combination of commands"),
    }

    Ok(())
}
