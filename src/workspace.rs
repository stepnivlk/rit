use crate::errors::RitError;
use pathdiff::diff_paths;
use std::path::Path;
use std::{
    fmt,
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

#[derive(Debug, Clone)]
pub struct Entry {
    pub relative_path: PathBuf,
    pub absolute_path: PathBuf,
    pub name: String,
    pub relative_path_name: String,
    pub len: usize,
    pub is_dir: bool,
}

impl Entry {
    pub fn new(absolute_path: PathBuf, relative_path: PathBuf) -> Self {
        let relative_path_name: String = relative_path.to_string_lossy().into();
        let len = relative_path_name.len();
        let is_dir = absolute_path.is_dir();

        Self {
            name: relative_path.file_name().unwrap().to_string_lossy().into(),
            relative_path_name,
            relative_path,
            absolute_path,
            len,
            is_dir,
        }
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}",
            self.relative_path_name,
            if self.is_dir { "/" } else { "" }
        )
    }
}

impl Workspace {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn list_files(&self, path: Option<&PathBuf>) -> Vec<Entry> {
        let path = path.unwrap_or(&self.path);

        if path.is_dir() {
            self.read_dir(path)
                .flat_map(|entry| self.list_files(Some(&path.join(entry))))
                .collect()
        } else {
            let relative_path = diff_paths(&path, &self.path).unwrap();

            vec![Entry::new(path.clone(), relative_path)]
        }
    }

    pub fn list_dir(&self, path: Option<&PathBuf>) -> Vec<Entry> {
        let path = path.unwrap_or(&self.path);

        self.read_dir(path)
            .map(|path| {
                let relative_path = diff_paths(&path, &self.path).unwrap();

                Entry::new(path, relative_path)
            })
            .collect()
    }

    pub fn read_file(&self, entry: &Entry) -> Result<File, RitError> {
        OpenOptions::new()
            .read(true)
            .append(true)
            .open(&entry.absolute_path)
            .map_err(|err| match err.kind() {
                std::io::ErrorKind::PermissionDenied => {
                    RitError::PermissionDenied(entry.relative_path_name.clone())
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
        let path = fs::canonicalize(&self.path.join(pathname));

        match path {
            Ok(path) => Ok(path),
            Err(_) => Err(RitError::MissingFile(String::from(pathname))),
        }
    }

    fn read_dir(&self, path: &Path) -> impl Iterator<Item = PathBuf> {
        path.read_dir()
            .unwrap()
            .filter(|path| path.is_ok())
            .map(|path| path.unwrap().path())
            .filter(|path| {
                !matches!(
                    path.file_name().and_then(|f| f.to_str()),
                    Some(".git") | Some(".gitignore") | Some("target")
                )
            })
    }
}
