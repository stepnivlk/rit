use pathdiff::diff_paths;
use std::{
    fs::{File, OpenOptions},
    io::Error,
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::PathBuf,
};

pub struct Workspace<'a> {
    path: &'a PathBuf,
}

#[derive(Debug, Clone)]
pub struct Metadata {
    pub ctime: i64,
    pub ctime_nsec: i64,
    pub mtime: i64,
    pub mtime_nsec: i64,
    pub dev: u64,
    pub ino: u64,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub size: u64,
}

#[derive(Debug)]
pub struct Stat {
    pub is_executable: bool,
    pub metadata: Metadata,
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
        let metadata = file.metadata().unwrap();
        let mode = metadata.permissions().mode();

        Stat {
            is_executable: mode & 0o111 != 0,
            metadata: Metadata {
                ctime: metadata.ctime(),
                ctime_nsec: metadata.ctime_nsec(),
                mtime: metadata.mtime(),
                mtime_nsec: metadata.mtime_nsec(),
                dev: metadata.dev(),
                ino: metadata.ino(),
                mode: metadata.mode(),
                uid: metadata.uid(),
                gid: metadata.gid(),
                size: metadata.size(),
            },
        }
    }
}
