use crate::{id, objects};
use libflate::zlib::{Encoder, Decoder};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use std::{
    fs::{File, OpenOptions},
    io::{self, prelude::*},
    path::{Path, PathBuf},
};

pub struct Database {
    path: PathBuf,
}

impl Database {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn store<O>(&self, object: &mut O) -> Result<id::Id, io::Error>
    where
        O: objects::Storable,
    {
        let id = object.store(|(id, data)| {
            self.write_object(id, data).unwrap();
        })?;

        Ok(id)
    }

    pub fn load(&self, id: &str) {
        self.read_object(id);
    }

    fn read_object(&self, id: &str) {
        let (_, object_path) = self.get_object_paths(id);

        let bytes = std::fs::read(object_path).unwrap();

        let mut decoder = Decoder::new(&bytes[..]).unwrap();

        let mut buf = Vec::new();

        decoder.read_to_end(&mut buf).unwrap();

        let mut decoded_iter = buf.iter();

        let mut object_type: Vec<u8> = vec![];
        let mut size: Vec<u8> = vec![];

        while let Some(byte) = decoded_iter.next() {
            if byte != &b' ' {
                object_type.push(*byte);
            } else {
                break;
            }
        }

        while let Some(byte) = decoded_iter.next() {
            if byte != &b'\0' {
                size.push(*byte);
            } else {
                break;
            }
        }

        let object_type = String::from_utf8_lossy(&object_type);
        dbg!(&object_type);
        dbg!(&size);

        match &object_type[..] {
            "commit" => {
                let obj = objects::Commit::from(decoded_iter);
            }
            _ => panic!()
        }
    }

    fn write_object<C: Read>(&self, id: &str, mut content: C) -> Result<(), io::Error> {
        let (dir_path, object_path) = self.get_object_paths(id);

        if object_path.exists() {
            return Ok(());
        }

        let temp_path = dir_path.join(self.generate_temp_name());

        let file = match self.open_temp_file(&temp_path) {
            Ok(file) => Ok(file),
            Err(_) => {
                std::fs::create_dir_all(dir_path)?;
                self.open_temp_file(&temp_path)
            }
        }?;

        let mut encoder = Encoder::new(file)?;
        std::io::copy(&mut content, &mut encoder)?;
        encoder.finish().into_result()?;

        std::fs::rename(temp_path, object_path)?;

        Ok(())
    }

    fn open_temp_file(&self, path: &Path) -> Result<File, io::Error> {
        OpenOptions::new().write(true).create_new(true).open(path)
    }

    fn generate_temp_name(&self) -> String {
        let rng = thread_rng();
        let s: String = rng.sample_iter(Alphanumeric).take(6).collect();

        format!("tmp_obj_{}", s)
    }

    fn get_object_paths(&self, id: &str) -> (PathBuf, PathBuf) {
        let dir_path = self.path.join(&id[0..2]);
        let object_path = dir_path.join(&id[2..]);

        (dir_path, object_path)
    }
}
