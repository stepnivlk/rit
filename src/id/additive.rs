use super::Id;
use sha1::Sha1;
use std::io::prelude::*;

#[derive(Clone)]
pub struct Additive(Sha1);

impl Additive {
    pub fn new() -> Self {
        Self(Sha1::new())
    }

    pub fn add<D: Read>(&mut self, mut data: D) {
        let mut buffer = [0; 1024];

        loop {
            let count = data.read(&mut buffer).unwrap();

            if count == 0 {
                break;
            }

            self.0.update(&buffer[..count]);
        }
    }

    pub fn commit(&mut self) -> Id {
        let id = Id {
            as_bytes: self.0.digest().bytes(),
            as_str: self.0.hexdigest(),
        };

        self.0 = Sha1::new();

        id
    }
}
