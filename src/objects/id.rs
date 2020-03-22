use sha1::Sha1;
use std::io::prelude::*;

#[derive(Clone)]
pub struct Id {
    pub as_str: String,
    pub as_bytes: [u8; 20],
}

impl Id {
    pub fn new<D: Read>(mut data: D) -> Self {
        let mut hasher = Sha1::new();
        let mut buffer = [0; 1024];

        loop {
            let count = data.read(&mut buffer).unwrap();

            if count == 0 {
                break;
            }

            hasher.update(&buffer[..count]);
        }

        Self {
            as_bytes: hasher.digest().bytes(),
            as_str: hasher.hexdigest(),
        }
    }
}
