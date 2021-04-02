use crate::{
    id,
    workspace::{Metadata, Stat},
};
use bytes::{BufMut, Bytes, BytesMut};

const REGULAR_MODE: u32 = 0o100644;
const EXECUTABLE_MODE: u32 = 0o100755;
const MAX_PATH_SIZE: usize = 0xfff;

#[derive(Debug, Clone)]
pub struct Entry {
    id: id::Id,
    path: String,
    metadata: Metadata,
    mode: u32,
    flags: usize,
}

impl Entry {
    pub fn new(path: String, id: id::Id, stat: Stat) -> Self {
        // TODO:
        let path_size = path.len();

        Self {
            id,
            path,
            metadata: stat.metadata,
            mode: if stat.is_executable {
                EXECUTABLE_MODE
            } else {
                REGULAR_MODE
            },
            flags: if path_size < MAX_PATH_SIZE {
                path_size
            } else {
                MAX_PATH_SIZE
            },
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

        let path = format!("{}\0", self.path);
        buf.put(path.as_bytes());

        while buf.len() % 8 != 0 {
            buf.put_u8(0);
        }

        buf.freeze()
    }
}
