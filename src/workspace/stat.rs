#[derive(Debug, Clone)]
pub struct Stat {
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

impl Stat {
    pub fn is_executable(&self) -> bool {
        self.mode & 0o111 != 0
    }
}
