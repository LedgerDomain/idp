use crate::{ContentMetadata, Hashable, ContentType};
use anyhow::Result;

impl ContentMetadata {
    pub fn content_type(&self) -> Result<ContentType> {
        ContentType::derive_from(&self.content_class, &self.content_format)
    }
}

impl Hashable for ContentMetadata {
    fn update_hasher(&self, hasher: &mut sha2::Sha256) {
        // NOTE: The specific order and form of this hashing must NOT be changed!
        self.content_length.update_hasher(hasher);
        self.content_class.update_hasher(hasher);
        self.content_format.update_hasher(hasher);
        self.content_encoding.update_hasher(hasher);
    }
}
