pub mod parsing;
pub mod fs;
pub mod logging;

#[cfg(test)]
mod test;

use std::slice::Iter;
use uuid::Uuid;

pub trait EnumIterator<T> {
    fn iterator() -> Iter<'static, T>;
}
pub struct InternalIdTooling;
impl InternalIdTooling {
    pub fn new_id() -> Uuid {
        Uuid::new_v4()
    }
    pub fn new_compact_id() -> String {
        InternalIdTooling::new_id().simple().encode_lower(&mut Uuid::encode_buffer()).to_string()
    }
}

