use crate::{Hashable, Nonce};

impl Nonce {
    #[cfg(feature = "nonce-generate")]
    pub fn generate() -> Self {
        let byte_count = 32;
        let mut byte_v = Vec::with_capacity(byte_count);
        byte_v.resize(byte_count, 0u8);
        let mut rng = rand::thread_rng();
        use rand::Rng;
        rng.fill(byte_v.as_mut_slice());
        Self { value: byte_v }
    }
}

impl std::fmt::Display for Nonce {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        for byte in &self.value {
            write!(f, "{:02X}", byte)?
        }
        Ok(())
    }
}

impl Hashable for Nonce {
    fn update_hasher(&self, hasher: &mut sha2::Sha256) {
        self.value.update_hasher(hasher);
    }
}
