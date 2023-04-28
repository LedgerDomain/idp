use crate::{Hashable, Seal};

impl std::fmt::Display for Seal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.sha256sum)
    }
}

impl Hashable for Seal {
    fn update_hasher(&self, hasher: &mut sha2::Sha256) {
        self.value.update_hasher(hasher);
    }
}
