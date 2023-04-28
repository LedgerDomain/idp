use crate::{Content, Hashable};

impl Hashable for Content {
    fn update_hasher(&self, hasher: &mut sha2::Sha256) {
        // NOTE: The specific order and form of this hashing must NOT be changed!
        self.content_metadata.update_hasher(hasher);
        self.content_byte_v.update_hasher(hasher);
    }
}
