use rand::{distributions::Alphanumeric, thread_rng, Rng};
use rit::errors::RitError;
use std::{
    fs::{self, OpenOptions},
    io::prelude::*,
    os::unix::fs::PermissionsExt,
    panic,
    path::PathBuf,
};

pub struct Project {
    pub dir: PathBuf,
    session: rit::Session,
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
        let s: String = rng.sample_iter(Alphanumeric).take(10).collect();

        PathBuf::from(format!("./tests/testdir/tmp_dir_{}", s))
    }

    fn get_session() -> rit::Session {
        let name = String::from("name");
        let email = String::from("email");

        rit::Session::new(Some(name), Some(email)).unwrap()
    }

    fn new() -> Self {
        let dir = Self::get_dir();
        let session = Self::get_session();

        let args = vec!["init".to_string()];

        rit::execute(rit::CommandOpts {
            dir: dir.clone(),
            session: session.clone(),
            args,
        })
        .unwrap();

        Self { dir, session }
    }

    pub fn cmd(&self, args: Vec<&str>) -> Result<rit::Execution, RitError> {
        rit::execute(rit::CommandOpts {
            dir: self.dir.clone(),
            session: self.session.clone(),
            args: args.iter().map(|arg| arg.to_string()).collect(),
        })
    }

    pub fn write_file(&self, name: &str, content: &str) {
        let path = self.dir.join(name);
        let prefix = path.parent().unwrap();
        fs::create_dir_all(prefix).unwrap();

        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .unwrap();

        file.write_all(content.as_bytes()).unwrap();
    }

    pub fn index_entries(&self) -> Vec<(String, u32)> {
        let mut repo = rit::Repository::new(self.dir.clone());
        repo.index.load().unwrap();

        repo.index
            .entries()
            .iter()
            .map(|entry| (entry.pathname.clone(), entry.mode))
            .collect()
    }

    pub fn expected_path(&self, name: &str) -> String {
        format!(
            "{}/{}",
            self.dir.canonicalize().unwrap().to_str().unwrap(),
            name
        )
        .to_string()
    }

    pub fn make_executable(&self, name: &str) {
        self.set_file_mode(name, 0o755);
    }

    pub fn make_unreadable(&self, name: &str) {
        self.set_file_mode(name, 0);
    }

    fn set_file_mode(&self, name: &str, mode: u32) {
        let path = self.dir.join(name);
        let mut perms = fs::metadata(&path).unwrap().permissions();

        perms.set_mode(mode);

        fs::set_permissions(path, perms).unwrap();
    }

    fn close(&self) {
        fs::remove_dir_all(&self.dir).unwrap();
    }
}
