use std::fs;
use std::path::PathBuf;

mod common;

fn get_entry_names(dir: PathBuf) -> Vec<String> {
    fs::read_dir(dir)
        .unwrap()
        .map(|entry| {
            entry
                .unwrap()
                .path()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string()
        })
        .collect()
}

#[test]
fn it_creates_default_structure_in_current_directory() {
    common::Project::open_clean(|project| {
        project.init(None).unwrap();

        let entries = get_entry_names(project.dir().join(".git"));

        assert_eq!(vec!["objects", "refs"], entries);
    });
}

#[test]
fn it_creates_default_structure_in_specified_directory() {
    common::Project::open_clean(|project| {
        project.init(Some("custom")).unwrap();

        let entries = get_entry_names(project.dir().join("custom/.git"));

        assert_eq!(vec!["objects", "refs"], entries);
    });
}
