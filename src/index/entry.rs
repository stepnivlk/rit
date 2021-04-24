use super::{bytes_to_uint, bytes_to_uint16};
use crate::{id, workspace};
use bytes::{BufMut, Bytes, BytesMut};
use std::path::PathBuf;

const REGULAR_MODE: u32 = 0o100644;
const EXECUTABLE_MODE: u32 = 0o100755;
const MAX_PATH_SIZE: usize = 0xfff;

#[derive(Debug, Clone)]
pub struct Entry {
    pub id: id::Id,
    pub path: PathBuf,
    pub pathname: String,
    metadata: workspace::Metadata,
    pub mode: u32,
    flags: usize,
}

impl Entry {
    pub fn new(workspace_entry: workspace::Entry, id: id::Id, stat: workspace::Stat) -> Self {
        Self {
            id,
            path: workspace_entry.relative_path,
            pathname: workspace_entry.relative_path_name,
            metadata: stat.metadata,
            mode: if stat.is_executable {
                EXECUTABLE_MODE
            } else {
                REGULAR_MODE
            },
            flags: if workspace_entry.len < MAX_PATH_SIZE {
                workspace_entry.len
            } else {
                MAX_PATH_SIZE
            },
        }
    }

    pub fn matches_stat(&self, stat: &workspace::Stat) -> bool {
        self.metadata.size == stat.metadata.size
    }

    pub fn parents(&self) -> Vec<PathBuf> {
        let mut path = self.path.iter();
        let mut expanded_parent_path = PathBuf::new();

        path.next_back();

        let mut parents = vec![];

        for parent in path {
            expanded_parent_path.push(parent);
            parents.push(expanded_parent_path.clone());
        }

        parents
    }

    pub fn parse(data: Vec<u8>) -> Self {
        let ctime = bytes_to_uint(&data[..4]);
        let ctime_nsec = bytes_to_uint(&data[4..8]);
        let mtime = bytes_to_uint(&data[8..12]);
        let mtime_nsec = bytes_to_uint(&data[12..16]);
        let dev = bytes_to_uint(&data[16..20]);
        let ino = bytes_to_uint(&data[20..24]);
        let mode = bytes_to_uint(&data[24..28]);
        let uid = bytes_to_uint(&data[28..32]);
        let gid = bytes_to_uint(&data[32..36]);
        let size = bytes_to_uint(&data[36..40]);
        let id = id::Id::parse(&data[40..60]);
        let flags = bytes_to_uint16(&data[60..62]);

        let mut path: Vec<u8> = vec![];
        let mut pos = 62;

        while data[pos] != 0x00 {
            path.push(data[pos]);

            pos += 1;
        }

        let pathname = String::from_utf8_lossy(&path).to_string();

        Self {
            id,
            path: PathBuf::from(pathname.clone()),
            pathname,
            metadata: workspace::Metadata {
                ctime: ctime.into(),
                ctime_nsec: ctime_nsec.into(),
                mtime: mtime.into(),
                mtime_nsec: mtime_nsec.into(),
                dev: dev.into(),
                ino: ino.into(),
                mode,
                uid,
                gid,
                size: size.into(),
            },
            mode,
            flags: flags.into(),
        }
    }

    pub fn data(&self) -> Bytes {
        let mut buf = BytesMut::new();

        buf.put_u32(self.metadata.ctime as u32);
        buf.put_u32(self.metadata.ctime_nsec as u32);
        buf.put_u32(self.metadata.mtime as u32);
        buf.put_u32(self.metadata.mtime_nsec as u32);
        buf.put_u32(self.metadata.dev as u32);
        buf.put_u32(self.metadata.ino as u32);
        buf.put_u32(self.mode);
        buf.put_u32(self.metadata.uid);
        buf.put_u32(self.metadata.gid);
        buf.put_u32(self.metadata.size as u32);
        buf.put(&self.id.as_bytes[..]);
        buf.put_u16(self.flags as u16);

        let pathname = format!("{}\0", self.pathname);
        buf.put(pathname.as_bytes());

        while buf.len() % 8 != 0 {
            buf.put_u8(0);
        }

        buf.freeze()
    }
}
