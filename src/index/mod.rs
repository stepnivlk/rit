mod checksum;
mod entry;
mod errors;
mod index;

use checksum::Checksum;
pub use entry::Entry;
pub use errors::IndexError;
pub use index::{Index, IndexIter};

fn bytes_to_uint32(bytes: &[u8]) -> u32 {
    let mut num = [0u8; 4];
    num.clone_from_slice(bytes);

    u32::from_be_bytes(num)
}

fn bytes_to_uint16(bytes: &[u8]) -> u16 {
    let mut num = [0u8; 2];
    num.clone_from_slice(bytes);

    u16::from_be_bytes(num)
}
