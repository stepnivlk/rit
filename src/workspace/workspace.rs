use super::{Entry, Stat};
use crate::errors::RitError;
use pathdiff::diff_paths;
use std::{
    fs::{self, File, OpenOptions},
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

pub struct Workspace {
    path: PathBuf,
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

    pub fn list_dir(&self, path: Option<&PathBuf>) -> Vec<(Entry, Stat)> {
        let path = path.unwrap_or(&self.path);

        self.read_dir(path)
            .map(|path| {
                let relative_path = diff_paths(&path, &self.path).unwrap();

                let file = File::open(&path).unwrap();
                let stat = self.stat_file(&file);

                (Entry::new(path, relative_path), stat)
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

        Stat {
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
                    Some(".git") | Some("target")
                )
            })
    }
}
