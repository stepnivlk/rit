use io::Read;
use std::{env, fs, io, path::Path};

mod objects;

mod workspace;
use workspace::Workspace;

mod database;
use database::Database;

mod errors;
use errors::RitError;

mod refs;
use refs::Refs;

mod lockfile;

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
    let db_path = git_path.join("objects");

    let path = path.to_path_buf();

    let workspace = Workspace::new(&path);
    let database = Database::new(&db_path);
    let refs = Refs::new(&git_path);

    let entries: Vec<objects::Entry> = workspace
        .list_files(None)
        .into_iter()
        .map(|entry| {
            let file = workspace.read_file(&entry).unwrap();
            let stat = workspace.stat_file(&file);

            let mut blob = objects::Blob::new(file);

            let blob_id = database.store(&mut blob).unwrap();

            objects::Entry {
                id: blob_id,
                path: entry,
                stat,
            }
        })
        .collect();

    let mut root = objects::Tree::build(entries);

    root.traverse(|tree| {
        let id = database.store(tree).unwrap();
        tree.id = Some(id);
    });

    let root_id = database.store(&mut root)?;

    let parent = refs.read_head();
    let (name, email, message) = env_data()?;

    let author = objects::Author::new(name, email);

    let mut commit = objects::Commit::new(&parent, root_id, author, &message);
    let commit_id = database.store(&mut commit)?;
    refs.update_head(&commit_id)?;

    let root_part = match parent {
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
