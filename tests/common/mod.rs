#![allow(dead_code)]

use rand::{distributions::Alphanumeric, thread_rng, Rng};
use rit::{errors::RitError, Command, Session};
use std::{
    fs::{self, OpenOptions},
    io::prelude::*,
    os::unix::fs::PermissionsExt,
    panic,
    path::PathBuf,
};

pub struct Project {
    session: Session,
}

impl Project {
    pub fn open<T>(test: T) -> ()
    where
        T: FnOnce(&Self) -> () + panic::UnwindSafe,
    {
        let project = Self::new();

        let result = panic::catch_unwind(|| test(&project));

        project.close();

        assert!(result.is_ok())
    }

    fn get_dir() -> PathBuf {
        let rng = thread_rng();
        let name: String = rng.sample_iter(Alphanumeric).take(10).collect();
        let name = format!("./tests/testdir/tmp_dir_{}", name);

        let path = PathBuf::from(name);

        fs::create_dir(&path).unwrap();

        fs::canonicalize(path).unwrap()
    }

    fn new() -> Self {
        let project_dir = Self::get_dir();

        let author_name = String::from("name");
        let author_email = String::from("email");
        let session = Session {
            author_name,
            author_email,
            project_dir,
        };

        rit::Init::new(session.clone(), None).execute().unwrap();

        Self { session }
    }

    pub fn add(&self, paths: Vec<&str>) -> Result<rit::Execution, RitError> {
        let paths = paths.iter().map(|path| path.to_string()).collect();

        rit::Add::new(self.session.clone(), paths).execute()
    }

    pub fn commit(&self, message: &str) -> Result<rit::Execution, RitError> {
        rit::Commit::new(self.session.clone(), message.to_string()).execute()
    }

    pub fn status(&self) -> Result<rit::Execution, RitError> {
        rit::Status::new(self.session.clone()).execute()
    }

    pub fn write_file(&self, name: &str, content: &str) {
        let path = self.session.project_dir.join(name);
        let prefix = path.parent().unwrap();
        fs::create_dir_all(prefix).unwrap();

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(path)
            .unwrap();

        file.write_all(content.as_bytes()).unwrap();
    }

    pub fn index_entries(&self) -> Vec<(String, u32)> {
        let mut repo = rit::Repository::new(self.session.project_dir.clone());
        repo.index.load().unwrap();

        repo.index
            .entries()
            .iter()
            .map(|entry| (entry.pathname.clone(), entry.mode))
            .collect()
    }

    pub fn expected_path(&self, name: &str) -> String {
        name.to_string()
    }

    pub fn make_executable(&self, name: &str) {
        self.set_file_mode(name, 0o755);
    }

    pub fn make_unreadable(&self, name: &str) {
        self.set_file_mode(name, 0);
    }

    pub fn make_dir(&self, name: &str) {
        fs::create_dir(self.session.project_dir.join(name)).unwrap();
    }

    fn set_file_mode(&self, name: &str, mode: u32) {
        let path = self.session.project_dir.join(name);
        let mut perms = fs::metadata(&path).unwrap().permissions();

        perms.set_mode(mode);

        fs::set_permissions(path, perms).unwrap();
    }

    fn close(&self) {
        fs::remove_dir_all(&self.session.project_dir).unwrap();
    }
}
