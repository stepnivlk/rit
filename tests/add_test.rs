use rit::{errors::RitError, index::IndexError};

mod common;

#[test]
fn it_adds_regular_file_to_index() {
    common::Project::open(|project| {
        project.write_file("hello.txt", "hello");

        project.cmd(vec!["add", "hello.txt"]).unwrap();

        let expected_entries = vec![(project.expected_path("hello.txt"), 0o100644)];

        assert_eq!(expected_entries, project.index_entries());
    });
}

#[test]
fn it_adds_executable_file_to_index() {
    common::Project::open(|project| {
        project.write_file("hello.txt", "hello");
        project.make_executable("hello.txt");

        project.cmd(vec!["add", "hello.txt"]).unwrap();

        let expected_entries = vec![(project.expected_path("hello.txt"), 0o100755)];

        assert_eq!(expected_entries, project.index_entries());
    });
}

#[test]
fn it_adds_multiple_files_to_index() {
    common::Project::open(|project| {
        project.write_file("hello.txt", "hello");
        project.write_file("world.txt", "world");

        project.cmd(vec!["add", "hello.txt", "world.txt"]).unwrap();

        let expected_entries = vec![
            (project.expected_path("hello.txt"), 0o100644),
            (project.expected_path("world.txt"), 0o100644),
        ];

        assert_eq!(expected_entries, project.index_entries());
    });
}

#[test]
fn it_incrementally_adds_files_to_index() {
    common::Project::open(|project| {
        project.write_file("hello.txt", "hello");
        project.write_file("world.txt", "world");

        project.cmd(vec!["add", "world.txt"]).unwrap();

        assert_eq!(
            vec![(project.expected_path("world.txt"), 0o100644)],
            project.index_entries()
        );

        project.cmd(vec!["add", "hello.txt"]).unwrap();

        let expected_entries = vec![
            (project.expected_path("hello.txt"), 0o100644),
            (project.expected_path("world.txt"), 0o100644),
        ];

        assert_eq!(expected_entries, project.index_entries());
    });
}

#[test]
fn it_adds_directory_to_index() {
    common::Project::open(|project| {
        project.write_file("a-dir/nested.txt", "content");

        project.cmd(vec!["add", "a-dir"]).unwrap();

        assert_eq!(
            vec![(project.expected_path("a-dir/nested.txt"), 0o100644)],
            project.index_entries()
        );
    });
}

#[test]
fn it_adds_repository_root_to_index() {
    common::Project::open(|project| {
        project.write_file("a/b/c/file.txt", "content");

        project.cmd(vec!["add", "."]).unwrap();

        assert_eq!(
            vec![(project.expected_path("a/b/c/file.txt"), 0o100644)],
            project.index_entries()
        );
    });
}

#[test]
fn it_fails_for_nonexistent_file() {
    common::Project::open(|project| {
        match project.cmd(vec!["add", "no-such-file"]) {
            Err(RitError::MissingFile(_)) => assert!(true),
            _ => assert!(false, "MissingFile Err should be returned"),
        };
    });
}

#[test]
fn it_fails_for_unreadable_file() {
    common::Project::open(|project| {
        project.write_file("secret.txt", "");
        project.make_unreadable("secret.txt");

        match project.cmd(vec!["add", "secret.txt"]) {
            Err(RitError::PermissionDenied(_)) => assert!(true),
            _ => {
                assert!(false, "PermissionDenied Err should be returned");
            }
        };
    });
}

#[test]
fn it_fails_when_index_is_locked() {
    common::Project::open(|project| {
        project.write_file("file.txt", "");
        project.write_file(".git/index.lock", "");

        match project.cmd(vec!["add", "file.txt"]) {
            Err(RitError::Index(IndexError::Lock(_))) => assert!(true),
            _ => {
                assert!(false, "Index Lock Err should be returned");
            }
        }
    });
}
