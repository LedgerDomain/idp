use crate::{Hashable, PlumMetadata};

impl PlumMetadata {
    pub fn empty() -> Self {
        Self {
            plum_metadata_nonce_o: None,
            plum_created_at_o: None,
            plum_body_content_metadata_o: None,
            additional_content_o: None,
        }
    }
}

impl Hashable for PlumMetadata {
    fn update_hasher(&self, hasher: &mut sha2::Sha256) {
        // NOTE: The specific order and form of this hashing must NOT be changed!
        self.plum_metadata_nonce_o.update_hasher(hasher);
        self.plum_created_at_o.update_hasher(hasher);
        self.plum_body_content_metadata_o.update_hasher(hasher);
        self.additional_content_o.update_hasher(hasher);
    }
}
