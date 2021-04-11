use crate::errors::RitError;
use pathdiff::diff_paths;
use std::{
    fs::{self, File, OpenOptions},
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::PathBuf,
};

pub struct Workspace {
    path: PathBuf,
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

impl Workspace {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn list_files(&self, path: Option<&PathBuf>) -> Vec<PathBuf> {
        let path = path.unwrap_or(&self.path);

        if path.is_dir() {
            path.read_dir()
                .unwrap()
                .filter(|entry| entry.is_ok())
                .map(|entry| entry.unwrap().path())
                .filter(|entry| match entry.file_name().and_then(|f| f.to_str()) {
                    Some(".git") => false,
                    Some(".gitignore") => false,
                    Some("target") => false,
                    _ => true,
                })
                .flat_map(|entry| self.list_files(Some(&path.join(entry))))
                .collect()
        } else {
            let entry = diff_paths(&path, &self.path).unwrap();

            vec![entry]
        }
    }

    pub fn read_file(&self, path: &PathBuf) -> Result<File, RitError> {
        OpenOptions::new()
            .read(true)
            .append(true)
            .open(path)
            .map_err(|err| match err.kind() {
                std::io::ErrorKind::PermissionDenied => {
                    RitError::PermissionDenied(path.to_string_lossy().to_string())
                }
                _ => RitError::Io(err),
            })
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

    pub fn expand_path(&self, pathname: &str) -> Result<PathBuf, RitError> {
        let path = fs::canonicalize(pathname);

        match path {
            Ok(path) => Ok(path),
            Err(_) => Err(RitError::MissingFile(String::from(pathname))),
        }
    }
}
