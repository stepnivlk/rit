use std::{fmt, path::PathBuf};

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
