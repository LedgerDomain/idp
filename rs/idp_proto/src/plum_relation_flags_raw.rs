use crate::{Hashable, PlumRelationFlagsRaw};

impl Hashable for PlumRelationFlagsRaw {
    /// PlumRelationFlagsRaw is a u32, which is hashed in little-endian byte order.
    fn update_hasher(&self, hasher: &mut sha2::Sha256) {
        self.value.to_le_bytes().as_slice().update_hasher(hasher);
    }
}
