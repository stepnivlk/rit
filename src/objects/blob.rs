use crate::objects::{Object, Storable};
use bytes::{buf::BufMutExt, Bytes, BytesMut};
use std::{fmt, fs::File, io};

pub struct Blob {
    file: File,
}

impl Blob {
    pub fn new(file: File) -> Self {
        Self { file }
    }
}

impl fmt::Display for Blob {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "blob")
    }
}

impl Object for Blob {
    fn data(&mut self) -> Bytes {
        let mut data = BytesMut::new().writer();
        io::copy(&mut self.file, &mut data).unwrap();

        data.into_inner().freeze()
    }
}

impl Storable for Blob {}
