use crate::{Hashable, Plum, PlumMetadata, PlumMetadataSeal, Seal, Sha256Sum};

impl std::fmt::Display for PlumMetadataSeal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.value)
    }
}

impl From<&Plum> for PlumMetadataSeal {
    fn from(plum: &Plum) -> PlumMetadataSeal {
        PlumMetadataSeal::from(&plum.plum_metadata)
    }
}

impl From<&PlumMetadata> for PlumMetadataSeal {
    fn from(plum_metadata: &PlumMetadata) -> PlumMetadataSeal {
        use sha2::Digest;

        // For now, a seal is only the Sha256Sum, but it could be other stuff later.
        let mut hasher = sha2::Sha256::new();

        plum_metadata.update_hasher(&mut hasher);

        PlumMetadataSeal::from(Seal::from(Sha256Sum::from(hasher.finalize().to_vec())))
    }
}

impl From<Vec<u8>> for PlumMetadataSeal {
    fn from(byte_v: Vec<u8>) -> PlumMetadataSeal {
        if byte_v.len() != 32 {
            panic!("programmer error: PlumMetadataSeal must be 32 bytes long");
        }
        PlumMetadataSeal::from(Seal::from(Sha256Sum::from(byte_v)))
    }
}

impl Hashable for PlumMetadataSeal {
    fn update_hasher(&self, hasher: &mut sha2::Sha256) {
        self.value.update_hasher(hasher);
    }
}
