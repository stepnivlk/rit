mod common;

fn assert_changed(expected: Vec<&str>, execution: rit::Execution) {
    match execution {
        rit::Execution::Status(res) => {
            let names: Vec<String> = res
                .changed
                .iter()
                .map(|entry| format!("{}", entry))
                .collect();

            assert_eq!(expected, names);
        }
        _ => assert!(false),
    }
}

fn filled_project<T>(test: T)
where
    T: FnOnce(&common::Project) -> () + std::panic::UnwindSafe,
{
    common::Project::open(|project| {
        project.write_file("1.txt", "one");
        project.write_file("a/2.txt", "two");
        project.write_file("a/b/3.txt", "three");

        project.add(vec!["."]).unwrap();
        project.commit("message").unwrap();

        test(&project);
    });
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
