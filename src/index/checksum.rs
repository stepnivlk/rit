use super::IndexError;
use crate::id;
use std::{
    fs::File,
    io::{Error, Read},
};

pub struct Checksum {
    file: File,
    id_generator: id::Additive,
}

impl Checksum {
    pub fn new(file: File) -> Self {
        Self {
            file,
            id_generator: id::Additive::new(),
        }
    }

    pub fn read(&mut self, size: usize) -> Result<Vec<u8>, Error> {
        let mut buf = vec![0u8; size];

        self.file.read_exact(&mut buf).map(|_| {
            self.id_generator.add(&buf[..]);
        })?;

        Ok(buf)
    }

    pub fn verify_checksum(&mut self) -> Result<(), IndexError> {
        let mut stored_id = vec![0u8; 20];

        self.file.read_exact(&mut stored_id).unwrap();

        let generated_id = self.id_generator.commit().as_bytes;

        if stored_id != generated_id {
            return Err(IndexError::Parse(
                "Checksum verification of the index failed".to_string(),
            ));
        }

        Ok(())
    }
}
