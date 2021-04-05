use super::{bytes_to_uint, Checksum, Entry, IndexError};
use crate::{
    id,
    lockfile::{LockError, Lockfile},
    workspace::Stat,
};
use bytes::{BufMut, BytesMut};
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io,
    path::PathBuf,
};

const VERSION: u32 = 2;
const HEADER_SIZE: usize = 12;
const SIGNATURE: &[u8] = "DIRC".as_bytes();
const ENTRY_MIN_SIZE: usize = 64;

pub struct Index {
    lockfile: Lockfile,
    entries: HashMap<String, Entry>,
    id_builder: id::Additive,
    is_changed: bool,
}

impl Index {
    pub fn new(path: PathBuf) -> Self {
        Self {
            lockfile: Lockfile::new(path),
            entries: HashMap::new(),
            id_builder: id::Additive::new(),
            is_changed: false,
        }
    }

    pub fn load_for_update(&mut self) -> Result<(), IndexError> {
        self.lockfile.hold_for_update()?;

        self.load()
    }

    pub fn load(&mut self) -> Result<(), IndexError> {
        self.clear();

        let file = self.open_index_file();

        match file {
            Ok(f) => {
                let mut reader = Checksum::new(f);

                let count = self.read_header(&mut reader)?;
                self.read_entries(&mut reader, count)?;

                reader.verify_checksum()?;
            }
            Err(_) => {}
        };

        Ok(())
    }

    pub fn add(&mut self, path: PathBuf, id: id::Id, stat: Stat) {
        let entry = Entry::new(path, id, stat);

        self.entries.insert(entry.pathname.clone(), entry);
        self.is_changed = true;
    }

    pub fn entries(&self) -> Vec<Entry> {
        let mut entries = self
            .entries
            .iter()
            // TODO: -clone
            .map(|(_, entry)| entry.clone())
            .collect::<Vec<_>>();

        entries.sort_by_key(|entry| entry.pathname.clone());

        entries
    }

    pub fn write_updates(&mut self) -> Result<(), LockError> {
        if !self.is_changed {
            self.lockfile.rollback()?;

            return Ok(());
        }

        self.write_header()?;

        let entries = self.entries();

        for entry in entries {
            let data = &entry.data()[..];

            self.lockfile.write(&data[..])?;
            self.id_builder.add(&data[..]);
        }

        let id = self.id_builder.commit();

        self.lockfile.write(&id.as_bytes[..])?;
        self.lockfile.commit()?;

        self.is_changed = false;

        Ok(())
    }

    fn read_header(&self, reader: &mut Checksum) -> Result<u32, IndexError> {
        let data = reader.read(HEADER_SIZE)?;

        let signature = &data[..4];
        let version = bytes_to_uint(&data[4..8]);
        let count = bytes_to_uint(&data[8..12]);

        if signature != SIGNATURE {
            let msg = format!(
                "Signature: expected '{}' but found '{}'",
                String::from_utf8_lossy(SIGNATURE),
                String::from_utf8_lossy(signature)
            );

            return Err(IndexError::Parse(msg));
        };

        if version != VERSION {
            let msg = format!("Version: expected '{}' but found '{}'", VERSION, version,);

            return Err(IndexError::Parse(msg));
        }

        Ok(count)
    }

    fn read_entries(&mut self, reader: &mut Checksum, count: u32) -> Result<(), IndexError> {
        for _ in 0..count {
            let mut entry = reader.read(ENTRY_MIN_SIZE)?;

            while entry.last() != Some(&0x00) {
                let mut chunk = reader.read(8)?;

                entry.append(&mut chunk);
            }

            let entry = Entry::parse(entry);
            self.entries.insert(entry.pathname.clone(), entry);
        }

        Ok(())
    }

    fn clear(&mut self) {
        self.entries = HashMap::new();
        self.id_builder = id::Additive::new();
        self.is_changed = false;
    }

    fn open_index_file(&self) -> Result<File, io::Error> {
        OpenOptions::new().read(true).open(&self.lockfile.file_path)
    }

    fn write_header(&mut self) -> Result<(), LockError> {
        let mut buf = BytesMut::new();

        buf.put(SIGNATURE);
        buf.put_u32(VERSION);
        buf.put_u32(self.entries.len() as u32);

        self.lockfile.write(&buf[..])?;
        self.id_builder.add(&buf[..]);

        Ok(())
    }
}
