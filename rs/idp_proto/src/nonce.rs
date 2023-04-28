use crate::{Hashable, Nonce};

impl Hashable for Nonce {
    fn update_hasher(&self, hasher: &mut sha2::Sha256) {
        self.value.update_hasher(hasher);
    }
}
