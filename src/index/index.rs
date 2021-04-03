use super::Entry;
use crate::{
    id,
    lockfile::{LockError, Lockfile},
    workspace::Stat,
};
use bytes::{BufMut, BytesMut};
use std::{collections::HashMap, path::PathBuf};

const VERSION: u32 = 2;

pub struct Index {
    lockfile: Lockfile,
    entries: HashMap<String, Entry>,
    id_builder: id::Additive,
}

impl Index {
    pub fn new(path: PathBuf) -> Self {
        Self {
            lockfile: Lockfile::new(path),
            entries: HashMap::new(),
            id_builder: id::Additive::new(),
        }
    }

    pub fn add(&mut self, path: PathBuf, id: id::Id, stat: Stat) {
        let pathname = path.as_os_str().to_str().unwrap().to_string();
        let entry = Entry::new(pathname.clone(), id, stat);

        self.entries.insert(pathname, entry);
    }

    pub fn write_updates(&mut self) -> Result<(), LockError> {
        self.lockfile.hold_for_update()?;

        self.write_header()?;

        let mut entries = self.entries.iter().collect::<Vec<_>>();

        entries.sort_by_key(|entry| entry.0);

        for (_, entry) in &entries {
            let data = &entry.data()[..];

            self.lockfile.write(&data[..])?;
            self.id_builder.add(&data[..]);
        }

        let id = self.id_builder.commit();

        self.lockfile.write(&id.as_bytes[..])?;
        self.lockfile.commit()?;

        Ok(())
    }

    fn write_header(&mut self) -> Result<(), LockError> {
        let mut buf = BytesMut::new();

        buf.put("DIRC".as_bytes());
        buf.put_u32(VERSION);
        buf.put_u32(self.entries.len() as u32);

        self.lockfile.write(&buf[..])?;
        self.id_builder.add(&buf[..]);

        Ok(())
    }
}
