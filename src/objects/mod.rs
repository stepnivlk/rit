use bytes::Bytes;
use chrono::{DateTime, Local};
use std::{
    fmt,
    io::{prelude::*, Chain, Error},
};

pub mod commit;
pub use commit::Commit;

pub mod tree;
pub use tree::Tree;

pub mod blob;
pub use blob::Blob;

use crate::id;

pub trait Object: fmt::Display {
    fn data(&mut self) -> Bytes;

    fn header(&self, data_len: usize) -> String {
        format!("{} {}\0", self, data_len)
    }
}

pub type Data<'a> = (&'a str, Chain<&'a [u8], &'a [u8]>);

pub trait Storable: Object {
    fn store<W>(&mut self, writer: W) -> Result<id::Id, Error>
    where
        W: FnOnce(Data),
    {
        let data = &self.data()[..];

        let header = self.header(data.len());
        let header = header.as_bytes();

        let id = id::OneOff::new(header.chain(data));

        let data = header.chain(data);

        writer((&id.as_str[..], data));

        Ok(id)
    }
}

pub struct Author<'a> {
    name: &'a str,
    email: &'a str,
    time: DateTime<Local>,
}

impl<'a> Author<'a> {
    pub fn new(name: &'a str, email: &'a str) -> Self {
        Self {
            name,
            email,
            time: Local::now(),
        }
    }
}

impl<'a> fmt::Display for Author<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let timestamp = self.time.format("%s %z");

        write!(f, "{} <{}> {}", self.name, self.email, timestamp)
    }
}
