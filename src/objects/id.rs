use sha1::Sha1;
use std::{fmt, io::prelude::*};

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

impl fmt::Debug for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str)
    }
}
