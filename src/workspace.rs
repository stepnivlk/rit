use std::fs::File;
use std::fs::OpenOptions;
use std::io::Error;
use std::path::{Path, PathBuf};

pub struct Workspace<'a> {
    path: &'a Path,
}

impl<'a> Workspace<'a> {
    pub fn new(path: &'a Path) -> Self {
        Self { path }
    }

    pub fn list_files(&self) -> Result<impl Iterator<Item = PathBuf>, Error> {
        let contents = self.path.read_dir()?;

        let contents = contents
            .filter(|entry| entry.is_ok())
            .map(|entry| entry.unwrap().path())
            .filter(|entry| {
                if entry.is_dir() {
                    return false;
                }

                match entry.file_name().and_then(|f| f.to_str()) {
                    Some(".git") => false,
                    Some(".gitignore") => false,
                    _ => true,
                }
            });

        Ok(contents)
    }

    pub fn read_file(&self, path: &PathBuf) -> Result<File, Error> {
        OpenOptions::new().read(true).append(true).open(path)
    }
}
