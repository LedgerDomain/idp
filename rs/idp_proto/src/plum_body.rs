use crate::{Hashable, PlumBody};

impl Hashable for PlumBody {
    fn update_hasher(&self, hasher: &mut sha2::Sha256) {
        // NOTE: The specific order and form of this hashing must NOT be changed!
        self.plum_body_nonce_o.update_hasher(hasher);
        self.plum_body_content.update_hasher(hasher);
    }
}
