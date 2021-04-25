use super::{bytes_to_uint32, Checksum, Entry, IndexError};
use crate::{
    id,
    lockfile::{LockError, Lockfile},
    workspace,
};
use bytes::{BufMut, Bytes, BytesMut};
use sorted_vec::SortedSet;
use std::{
    collections::{HashMap, HashSet},
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
    entry_path_names: SortedSet<String>,
    parents: HashMap<String, HashSet<String>>,
    id_builder: id::Additive,
    is_changed: bool,
}

pub struct IndexIter<'a> {
    entries: &'a HashMap<String, Entry>,
    entry_path_names: &'a SortedSet<String>,
    cur: usize,
}

impl<'a> Iterator for IndexIter<'a> {
    type Item = &'a Entry;

    fn next(&mut self) -> Option<&'a Entry> {
        let name = self.entry_path_names.get(self.cur);

        match name {
            Some(n) => {
                self.cur += 1;

                self.entries.get(n)
            }
            _ => None,
        }
    }
}

impl Index {
    pub fn new(path: PathBuf) -> Self {
        Self {
            lockfile: Lockfile::new(path),
            entries: HashMap::new(),
            entry_path_names: SortedSet::new(),
            parents: HashMap::new(),
            id_builder: id::Additive::new(),
            is_changed: false,
        }
    }

    pub fn iter(&mut self) -> IndexIter {
        IndexIter {
            entries: &self.entries,
            entry_path_names: &self.entry_path_names,
            cur: 0,
        }
    }

    pub fn load_for_update(&mut self) -> Result<(), IndexError> {
        self.lockfile.hold_for_update()?;

        self.load()
    }

    pub fn load(&mut self) -> Result<(), IndexError> {
        self.clear();

        let file = self.open_index_file();

        if let Ok(f) = file {
            let mut reader = Checksum::new(f)?;

            let entries_count = self.read_header(&mut reader)?;

            self.read_entries(&mut reader, entries_count)?;

            reader.verify_checksum()?;
        };

        Ok(())
    }

    pub fn is_tracked(&self, pathname: &str) -> bool {
        self.entries.contains_key(pathname) || self.parents.contains_key(pathname)
    }

    pub fn add(&mut self, workspace_entry: workspace::Entry, id: id::Id, stat: workspace::Stat) {
        let entry = Entry::new(workspace_entry, id, stat);

        self.discard_conflicts(&entry);

        self.add_parents(&entry);

        self.entry_path_names.insert(entry.pathname.clone());
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

        for entry in self.entries() {
            let data: Bytes = entry.into();
            let data = &data[..];

            self.lockfile.write(data)?;
            self.id_builder.add(data);
        }

        let id = self.id_builder.commit();

        self.lockfile.write(&id.as_bytes[..])?;
        self.lockfile.commit()?;

        self.is_changed = false;

        Ok(())
    }

    pub fn release_lock(&mut self) -> Result<(), LockError> {
        self.lockfile.rollback()
    }

    pub fn update_entry_stat(&mut self, pathname: &str, stat: &workspace::Stat) {
        if let Some(entry) = self.entries.get_mut(pathname) {
            entry.update_stat(stat);
            self.is_changed = true;
        }
    }

    fn discard_conflicts(&mut self, entry: &Entry) {
        let parents = entry.parents();

        for parent in parents {
            self.remove_entry(parent.to_str().unwrap());
        }

        let set = self.parents.get_mut(&entry.pathname);

        if let Some(set) = set {
            for child in set.clone().iter() {
                self.remove_entry(child);
            }
        };
    }

    fn remove_entry(&mut self, pathname: &str) {
        self.entry_path_names.remove_item(&pathname.to_string());
        let entry = self.entries.remove(pathname);

        if entry.is_none() {
            return;
        }

        let entry = entry.unwrap();

        for parent in entry.parents() {
            let dirname = parent.to_str().unwrap();

            let dir = self.parents.get_mut(dirname);

            if let Some(dir) = dir {
                dir.remove(&entry.pathname);

                if dir.is_empty() {
                    self.parents.remove(dirname);
                }
            }
        }
    }

    fn read_header(&self, reader: &mut Checksum) -> Result<u32, IndexError> {
        let data = reader.read(HEADER_SIZE)?;

        let signature = &data[..4];
        let version = bytes_to_uint32(&data[4..8]);
        let count = bytes_to_uint32(&data[8..12]);

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
            let mut bytes = reader.read(ENTRY_MIN_SIZE)?;

            while bytes.last() != Some(&0x00) {
                let mut chunk = reader.read(8)?;

                bytes.append(&mut chunk);
            }

            let entry = Entry::from(bytes);

            self.add_parents(&entry);

            self.entry_path_names.insert(entry.pathname.clone());
            self.entries.insert(entry.pathname.clone(), entry);
        }

        Ok(())
    }

    fn add_parents(&mut self, entry: &Entry) {
        for parent in entry.parents() {
            let parent_pathname: String = parent.to_string_lossy().into();

            match self.parents.get_mut(&parent_pathname) {
                Some(contents) => {
                    contents.insert(entry.pathname.clone());
                }
                None => {
                    let mut contents = HashSet::new();
                    contents.insert(entry.pathname.clone());

                    self.parents.insert(parent_pathname, contents);
                }
            };
        }
    }

    fn clear(&mut self) {
        self.entries = HashMap::new();
        self.parents = HashMap::new();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::Id;

    fn get_index() -> Index {
        let tmp_path = std::env::current_dir().unwrap().join("tmp/");
        let index_path = tmp_path.join("index");

        Index::new(index_path)
    }

    fn get_id() -> Id {
        Id::parse(&[
            81, 232, 127, 146, 48, 252, 159, 201, 222, 122, 167, 182, 52, 254, 15, 207, 73, 234,
            223, 164,
        ])
    }

    fn get_stat() -> workspace::Stat {
        workspace::Stat {
            ctime: 1,
            ctime_nsec: 2,
            mtime: 3,
            mtime_nsec: 4,
            dev: 5,
            ino: 6,
            mode: 7,
            uid: 8,
            gid: 9,
            size: 10,
        }
    }

    fn get_workspace_entry(path: &str) -> workspace::Entry {
        let absolute_path = PathBuf::from(path);
        let relative_path = PathBuf::from(path);

        workspace::Entry::new(absolute_path, relative_path)
    }

    fn map_entries(entries: &Vec<Entry>) -> Vec<&str> {
        entries
            .iter()
            .map(|entry| &entry.pathname[..])
            .collect::<Vec<&str>>()
    }

    #[test]
    fn it_adds_a_single_file() {
        let mut index = get_index();

        index.add(get_workspace_entry("alice.txt"), get_id(), get_stat());

        let entries = index.entries();
        let entries = map_entries(&entries);

        assert_eq!(vec!["alice.txt"], entries);
    }

    #[test]
    fn it_replaces_a_file_with_a_directory() {
        let mut index = get_index();

        index.add(get_workspace_entry("alice.txt"), get_id(), get_stat());
        index.add(get_workspace_entry("bob.txt"), get_id(), get_stat());

        index.add(
            get_workspace_entry("alice.txt/nested.txt"),
            get_id(),
            get_stat(),
        );

        let entries = index.entries();
        let entries = map_entries(&entries);

        assert_eq!(vec!["alice.txt/nested.txt", "bob.txt"], entries);
    }

    #[test]
    fn it_replaces_a_directory_with_a_file() {
        let mut index = get_index();

        index.add(get_workspace_entry("alice.txt"), get_id(), get_stat());
        index.add(get_workspace_entry("nested/bob.txt"), get_id(), get_stat());

        index.add(get_workspace_entry("nested"), get_id(), get_stat());

        let entries = index.entries();
        let entries = map_entries(&entries);

        assert_eq!(vec!["alice.txt", "nested"], entries);
    }

    #[test]
    fn it_recursively_replaces_a_directory_with_a_file() {
        let mut index = get_index();

        index.add(get_workspace_entry("alice.txt"), get_id(), get_stat());
        index.add(get_workspace_entry("nested/bob.txt"), get_id(), get_stat());
        index.add(
            get_workspace_entry("nested/inner/claire.txt"),
            get_id(),
            get_stat(),
        );

        index.add(get_workspace_entry("nested"), get_id(), get_stat());

        let entries = index.entries();
        let entries = map_entries(&entries);

        assert_eq!(vec!["alice.txt", "nested"], entries);
    }
}
