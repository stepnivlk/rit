use std::fmt;

#[derive(Clone)]
pub struct Id {
    pub as_str: String,
    pub as_bytes: [u8; 20],
}

impl fmt::Debug for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str)
    }
}
