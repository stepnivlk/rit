use crate::{
    id::Id,
    objects::{entry, Entry, Object, Storable},
};
use bytes::{BufMut, Bytes, BytesMut};
use indexmap::IndexMap;
use std::{ffi::OsStr, fmt};

#[derive(Debug)]
pub enum Node {
    Tree(Tree),
    Entry(Entry),
}

#[derive(Debug)]
pub struct Tree {
    pub nodes: IndexMap<String, Node>,
    pub id: Option<Id>,
}

impl Tree {
    pub fn build(mut entries: Vec<Entry>) -> Self {
        entries.sort_by(|a, b| a.path.as_os_str().cmp(&b.path.as_os_str()));
        let mut root = Tree::new();

        for entry in entries {
            let path = entry.path.clone();
            let mut path = path.iter();
            let name = path.next_back().unwrap();

            root.add_node(path, name, entry);
        }

        root
    }

    pub fn new() -> Self {
        Self {
            nodes: IndexMap::new(),
            id: None,
        }
    }

    pub fn traverse<C>(&mut self, consumer: C)
    where
        C: FnOnce(&mut Tree) + Copy,
    {
        for (_name, node) in &mut self.nodes {
            if let Node::Tree(tree) = node {
                tree.traverse(consumer);
            }
        }

        consumer(self);
    }

    pub fn mode(&self) -> &[u8] {
        entry::DIRECTORY_MODE
    }

    fn add_node<'a>(
        &mut self,
        mut path: impl Iterator<Item = &'a OsStr>,
        name: &'a OsStr,
        entry: Entry,
    ) {
        match path.next() {
            Some(part) => {
                let part = part.to_str().unwrap();

                if let Some(Node::Tree(tree)) = self.nodes.get_mut(part) {
                    tree.add_node(path, name, entry);
                } else {
                    let mut tree = Tree::new();
                    tree.add_node(path, name, entry);

                    self.nodes.insert(part.to_string(), Node::Tree(tree));
                }
            }
            None => {
                let name = name.to_str().unwrap().to_string();
                self.nodes.insert(name, Node::Entry(entry));
            }
        }
    }
}

impl fmt::Display for Tree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "tree")
    }
}

struct NodeInfo<'a> {
    name: String,
    id: [u8; 20],
    mode: &'a [u8],
}

impl<'a> NodeInfo<'a> {
    fn new(name: &'a str, node: &'a Node) -> Self {
        let name = format!("{}\0", name);

        match node {
            Node::Tree(tree) => Self {
                name,
                id: tree.id.as_ref().unwrap().as_bytes,
                mode: tree.mode(),
            },
            Node::Entry(entry) => Self {
                name,
                id: entry.id.as_bytes,
                mode: entry.mode(),
            },
        }
    }
}

impl Object for Tree {
    fn data(&mut self) -> Bytes {
        let mut buf = BytesMut::new();

        for (name, node) in &self.nodes {
            let node_info = NodeInfo::new(name, node);

            buf.put(node_info.mode);
            buf.put(node_info.name.as_bytes());
            buf.put(&node_info.id[..]);
        }

        buf.freeze()
    }
}

impl Storable for Tree {}
