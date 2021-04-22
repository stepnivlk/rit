mod common;

#[test]
fn it_lists_untracked_files_in_name_order() {
    common::Project::open(|project| {
        project.write_file("file.txt", "");
        project.write_file("another.txt", "");

        let res = project.cmd(vec!["status"]).unwrap();

        match res {
            rit::Execution::Status(res) => {
                let names: Vec<String> = res
                    .untracked
                    .iter()
                    .map(|entry| entry.pathname.clone())
                    .collect();

                assert_eq!(vec!["another.txt", "file.txt"], names);
            }
            _ => assert!(false),
        }
    });
}
