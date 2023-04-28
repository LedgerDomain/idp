use crate::{Hashable, Sha256Sum};

impl std::fmt::Display for Sha256Sum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        for byte in &self.value {
            write!(f, "{:02X}", byte)?
        }
        Ok(())
    }
}

impl Hashable for Sha256Sum {
    fn update_hasher(&self, hasher: &mut sha2::Sha256) {
        self.value.update_hasher(hasher);
    }
}
