use std::convert::TryInto;

#[derive(Clone, Debug)]
pub struct Id {
    // TODO: rename
    pub as_str: String,
    pub as_bytes: [u8; 20],
}

impl Id {
    pub fn parse(data: &[u8]) -> Self {
        let mut stringified = String::new();

        for byte in data.iter() {
            stringified.push_str(&format!("{:x?}", byte));
        }

        Self {
            as_str: stringified,
            as_bytes: data.try_into().unwrap(),
        }
    }
}
