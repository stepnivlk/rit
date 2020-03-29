use crate::{objects::Id, workspace::Stat};
use std::path::PathBuf;

const REGULAR_MODE: &[u8] = b"100644 ";
const EXECUTABLE_MODE: &[u8] = b"100755 ";
pub const DIRECTORY_MODE: &[u8] = b"40000 ";

#[derive(Debug)]
pub struct Entry {
    pub id: Id,
    pub path: PathBuf,
    pub stat: Stat,
}

impl Entry {
    pub fn mode(&self) -> &[u8] {
        if self.stat.is_executable {
            EXECUTABLE_MODE
        } else {
            REGULAR_MODE
        }
    }
}
