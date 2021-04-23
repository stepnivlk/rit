mod common;

fn assert_untracked(expected: Vec<&str>, execution: rit::Execution) {
    match execution {
        rit::Execution::Status(res) => {
            let names: Vec<String> = res
                .untracked
                .iter()
                .map(|entry| format!("{}", entry))
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

        assert_untracked(vec!["another.txt", "file.txt"], execution);
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

        assert_untracked(vec!["file.txt"], execution);
    });
}

#[test]
fn it_lists_untracked_directories_without_contents() {
    common::Project::open(|project| {
        project.write_file("file.txt", "");
        project.write_file("dir/another.txt", "");

        let execution = project.cmd(vec!["status"]).unwrap();

        assert_untracked(vec!["dir/", "file.txt"], execution);
    });
}

#[test]
fn it_lists_untracked_files_in_tracked_directories() {
    common::Project::open(|project| {
        project.write_file("a/b/inner.txt", "");
        project.cmd(vec!["add", "."]).unwrap();
        project.cmd(vec!["commit"]).unwrap();

        project.write_file("a/outer.txt", "");
        project.write_file("a/b/c/file.txt", "");

        let execution = project.cmd(vec!["status"]).unwrap();

        assert_untracked(vec!["a/b/c/", "a/outer.txt"], execution);
    });
}

#[test]
fn it_does_not_list_empty_untracked_directories() {
    common::Project::open(|project| {
        project.make_dir("outer");

        let execution = project.cmd(vec!["status"]).unwrap();

        assert_untracked(vec![], execution);
    });
}

#[test]
fn it_lists_untracked_directories_that_indirectly_contain_files() {
    common::Project::open(|project| {
        project.write_file("outer/inner/file.txt", "");

        let execution = project.cmd(vec!["status"]).unwrap();

        assert_untracked(vec!["outer/"], execution);
    });
}
