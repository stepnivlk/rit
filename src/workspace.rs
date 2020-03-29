use pathdiff::diff_paths;
use std::{
    fs::{File, OpenOptions},
    io::Error,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
};

pub struct Workspace<'a> {
    path: &'a PathBuf,
}

#[derive(Debug)]
pub struct Stat {
    pub is_executable: bool,
}

impl<'a> Workspace<'a> {
    pub fn new(path: &'a PathBuf) -> Self {
        Self { path }
    }

    pub fn list_files(&self, dir: Option<&PathBuf>) -> Vec<PathBuf> {
        let contents = dir.unwrap_or(self.path).read_dir().unwrap();

        let contents = contents
            .filter(|entry| entry.is_ok())
            .map(|entry| entry.unwrap().path())
            .filter(|entry| match entry.file_name().and_then(|f| f.to_str()) {
                Some(".git") => false,
                Some(".gitignore") => false,
                Some("target") => false,
                _ => true,
            })
            .flat_map(|entry| {
                if entry.is_dir() {
                    self.list_files(Some(&entry))
                } else {
                    let entry = diff_paths(&entry, &self.path).unwrap();

                    vec![entry]
                }
            });

        contents.collect()
    }

    pub fn read_file(&self, path: &PathBuf) -> Result<File, Error> {
        OpenOptions::new().read(true).append(true).open(path)
    }

    pub fn stat_file(&self, file: &File) -> Stat {
        let mode = file.metadata().unwrap().permissions().mode();

        Stat {
            is_executable: mode & 0o111 != 0,
        }
    }
}
