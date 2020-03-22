use crate::objects::{Entry, Object, Storable};
use bytes::{BufMut, Bytes, BytesMut};
use std::fmt;

const MODE: &[u8] = b"100644 ";

pub struct Tree {
    pub len: u64,
    pub entries: Vec<Entry>,
}

impl Tree {
    pub fn new(mut entries: Vec<Entry>) -> Self {
        entries.sort_by(|a, b| a.path.cmp(&b.path));

        Self { len: 0, entries }
    }
}

impl fmt::Display for Tree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "tree")
    }
}

impl Object for Tree {
    fn data(&mut self) -> Bytes {
        let mut buf = BytesMut::new();

        for entry in &self.entries {
            let name = entry.path.file_name().unwrap().to_str().unwrap();
            let name = format!("{}\0", name);
            let id = entry.id.as_bytes;

            buf.put(MODE);
            buf.put(name.as_bytes());
            buf.put(&id[..]);
        }

        buf.freeze()
    }
}

impl Storable for Tree {}
