mod common;

use common::filled_project;

fn assert_deleted(expected: Vec<&str>, execution: rit::Execution) {
    match execution {
        rit::Execution::Status(res) => {
            let names: Vec<String> = res
                .deleted
                .iter()
                .map(|entry| format!("{}", entry))
                .collect();

            assert_eq!(expected, names);
        }
        _ => assert!(false),
    }
}

#[test]
fn it_lists_deleted_files() {
    filled_project(|project| {
        project.delete("a/2.txt");

        let execution = project.status().unwrap();

        assert_deleted(vec!["a/2.txt"], execution);
    });
}

#[test]
fn it_lists_files_in_deleted_directories() {
    filled_project(|project| {
        project.delete("a");

        let execution = project.status().unwrap();

        assert_deleted(vec!["a/2.txt", "a/b/3.txt"], execution);
    });
}
