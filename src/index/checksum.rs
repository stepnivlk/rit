use super::{bytes_to_uint32, IndexError};
use crate::id;
use std::{
    fs::File,
    io::{Error, Read},
};

pub struct Checksum {
    file: File,
    file_len: u64,
    consumed_len: u64,
    id_generator: id::Additive,
}

impl Checksum {
    pub fn new(file: File) -> Result<Self, IndexError> {
        let file_len = file.metadata()?.len();

        Ok(Self {
            file,
            file_len,
            consumed_len: 0,
            id_generator: id::Additive::new(),
        })
    }

    pub fn read(&mut self, size: usize) -> Result<Vec<u8>, Error> {
        let mut buf = vec![0u8; size];

        self.file.read_exact(&mut buf).map(|_| {
            self.id_generator.add(&buf[..]);
        })?;

        self.consumed_len += size as u64;

        Ok(buf)
    }

    pub fn verify_checksum(&mut self) -> Result<(), IndexError> {
        self.consume_extension()?;

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

    fn consume_extension(&mut self) -> Result<(), IndexError> {
        while self.consumed_len < self.file_len - 20 {
            let _signature = self.read(4)?;

            let size = self.read(4)?;
            let size = bytes_to_uint32(&size[..]);

            self.read(size as usize)?;
        }

        Ok(())
    }
}
