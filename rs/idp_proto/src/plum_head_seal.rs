use crate::{Hashable, Plum, PlumHead, PlumHeadSeal, Seal, Sha256Sum};

impl std::fmt::Display for PlumHeadSeal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.value)
    }
}

impl From<&Plum> for PlumHeadSeal {
    fn from(plum: &Plum) -> PlumHeadSeal {
        PlumHeadSeal::from(&plum.plum_head)
    }
}

impl From<&PlumHead> for PlumHeadSeal {
    fn from(plum_head: &PlumHead) -> PlumHeadSeal {
        use sha2::Digest;

        // For now, a seal is only the Sha256Sum, but it could be other stuff later.
        let mut hasher = sha2::Sha256::new();

        plum_head.update_hasher(&mut hasher);

        PlumHeadSeal::from(Seal::from(Sha256Sum::from(hasher.finalize().to_vec())))
    }
}

impl From<Vec<u8>> for PlumHeadSeal {
    fn from(byte_v: Vec<u8>) -> PlumHeadSeal {
        if byte_v.len() != 32 {
            panic!("programmer error: PlumHeadSeal must be 32 bytes long");
        }
        PlumHeadSeal::from(Seal::from(Sha256Sum::from(byte_v)))
    }
}

impl Hashable for PlumHeadSeal {
    fn update_hasher(&self, hasher: &mut sha2::Sha256) {
        self.value.update_hasher(hasher);
    }
}
