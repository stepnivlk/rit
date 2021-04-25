use super::{Additive, Id};
use std::io::prelude::*;

pub struct OneOff;

impl OneOff {
    // TODO: Should return Self
    pub fn new<D: Read>(data: D) -> Id {
        let mut additive = Additive::new();

        additive.add(data);

        additive.commit()
    }
}
