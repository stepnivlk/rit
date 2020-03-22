use crate::objects;
use libflate::zlib::Encoder;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::fs::{File, OpenOptions};
use std::io::{prelude::*, Error};
use std::path::Path;

pub struct Database<'a> {
    path: &'a Path,
}

impl<'a> Database<'a> {
    pub fn new(path: &'a Path) -> Self {
        Self { path }
    }

    pub fn store<O>(&self, mut object: O) -> Result<objects::Id, Error>
    where
        O: objects::Storable,
    {
        let id = object.store(|(id, data)| {
            self.write_object(id, data).unwrap();
        })?;

        Ok(id)
    }

    fn write_object<C: Read>(&self, id: &str, mut content: C) -> Result<(), Error> {
        let dir_path = self.path.join(&id[0..2]);
        let object_path = dir_path.join(&id[2..]);
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

    fn open_temp_file(&self, path: &Path) -> Result<File, Error> {
        OpenOptions::new().write(true).create_new(true).open(path)
    }

    fn generate_temp_name(&self) -> String {
        let rng = thread_rng();
        let s: String = rng.sample_iter(Alphanumeric).take(6).collect();

        format!("tmp_obj_{}", s)
    }
}
