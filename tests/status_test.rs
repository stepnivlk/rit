mod common;

fn assert_status(expected: Vec<&str>, execution: rit::Execution) {
    match execution {
        rit::Execution::Status(res) => {
            let names: Vec<String> = res
                .untracked
                .iter()
                .map(|entry| entry.relative_path_name.clone())
                .collect();

            assert_eq!(expected, names);
        }
        _ => assert!(false),
    }
}

#[test]
fn it_lists_untracked_files_in_name_order() {
    common::Project::open(|project| {
        project.write_file("file.txt", "");
        project.write_file("another.txt", "");

        let execution = project.cmd(vec!["status"]).unwrap();

        assert_status(vec!["another.txt", "file.txt"], execution);
    });
}

#[test]
fn it_lists_files_as_untracked_when_they_are_not_in_index() {
    common::Project::open(|project| {
        project.write_file("committed.txt", "");
        project.cmd(vec!["add", "."]).unwrap();
        project.cmd(vec!["commit"]).unwrap();

        project.write_file("file.txt", "");

        let execution = project.cmd(vec!["status"]).unwrap();

        assert_status(vec!["file.txt"], execution);
    });
}
