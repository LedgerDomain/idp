use crate::{Hashable, PlumBody, PlumBodySeal, Seal, Sha256Sum};

impl std::fmt::Display for PlumBodySeal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.value)
    }
}

impl From<&PlumBody> for PlumBodySeal {
    fn from(plum_body: &PlumBody) -> PlumBodySeal {
        use sha2::Digest;

        // For now, a seal is only the Sha256Sum, but it could be other stuff later.
        let mut hasher = sha2::Sha256::new();

        plum_body.update_hasher(&mut hasher);

        PlumBodySeal::from(Seal::from(Sha256Sum::from(hasher.finalize().to_vec())))
    }
}

impl From<Vec<u8>> for PlumBodySeal {
    fn from(byte_v: Vec<u8>) -> PlumBodySeal {
        if byte_v.len() != 32 {
            panic!("programmer error: PlumBodySeal must be 32 bytes long");
        }
        PlumBodySeal::from(Seal::from(Sha256Sum::from(byte_v)))
    }
}

impl Hashable for PlumBodySeal {
    fn update_hasher(&self, hasher: &mut sha2::Sha256) {
        self.value.update_hasher(hasher);
    }
}
