use crate::{
    id::Id,
    objects::{Author, Object, Storable},
};
use bytes::{BufMut, Bytes, BytesMut};
use std::fmt;

pub struct Commit<'a> {
    parent: &'a Option<String>,
    tree_id: Id,
    author: Author,
    message: &'a str,
}

impl<'a> Commit<'a> {
    pub fn new(parent: &'a Option<String>, tree_id: Id, author: Author, message: &'a str) -> Self {
        Self {
            parent,
            tree_id,
            author,
            message,
        }
    }

    fn tree(&self) -> String {
        format!("tree {}\n", self.tree_id.as_str)
    }

    fn author(&self) -> String {
        format!("author {}\n", self.author)
    }

    fn commiter(&self) -> String {
        format!("commiter {}\n", self.author)
    }
}

impl<'a> fmt::Display for Commit<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "commit")
    }
}

impl<'a> Object for Commit<'a> {
    fn data(&mut self) -> Bytes {
        let mut buf = BytesMut::new();

        buf.put(self.tree().as_bytes());

        if let Some(parent) = self.parent {
            let parent = format!("parent {}\n", parent);
            buf.put(parent.as_bytes());
        };

        buf.put(self.author().as_bytes());
        buf.put(self.commiter().as_bytes());
        buf.put(&b"\n"[..]);
        buf.put(self.message.as_bytes());

        buf.freeze()
    }
}

impl<'a> Storable for Commit<'a> {}
