mod common;

use common::filled_project;

fn assert_changed(expected: Vec<&str>, execution: rit::Execution) {
    match execution {
        rit::Execution::Status(res) => {
            let names: Vec<String> = res
                .modified
                .iter()
                .map(|entry| format!("{}", entry))
                .collect();

            assert_eq!(expected, names);
        }
        _ => assert!(false),
    }
}

#[test]
fn it_is_empty_when_no_files_are_changed() {
    filled_project(|project| {
        let execution = project.status().unwrap();

        assert_changed(vec![], execution);
    });
}

#[test]
fn it_lists_entries_with_modified_contents() {
    filled_project(|project| {
        project.write_file("1.txt", "changed");
        project.write_file("a/2.txt", "modified");

        let execution = project.status().unwrap();

        assert_changed(vec!["1.txt", "a/2.txt"], execution);
    });
}

#[test]
fn it_lists_entries_with_changed_modes() {
    filled_project(|project| {
        project.make_executable("a/2.txt");

        let execution = project.status().unwrap();

        assert_changed(vec!["a/2.txt"], execution);
    });
}

#[test]
fn it_lists_modified_entries_with_unchanged_size() {
    filled_project(|project| {
        project.write_file("a/b/3.txt", "hello");

        let execution = project.status().unwrap();

        assert_changed(vec!["a/b/3.txt"], execution);
    });
}

#[test]
fn it_lists_nothing_when_the_file_is_touched() {
    filled_project(|project| {
        project.touch("1.txt");

        let execution = project.status().unwrap();

        assert_changed(vec![], execution);
    });
}
