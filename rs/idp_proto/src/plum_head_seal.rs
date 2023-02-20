use crate::{Plum, PlumHead, PlumHeadSeal, Seal, Sha256Sum};

impl AsRef<[u8]> for PlumHeadSeal {
    fn as_ref(&self) -> &[u8] {
        self.value.as_ref()
    }
}

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

        // NOTE: The specific order and form of this hashing must NOT be changed!

        if let Some(plum_head_nonce) = &plum_head.plum_head_nonce_o {
            hasher.update(b"\x01");
            hasher.update(&plum_head_nonce.value);
        } else {
            hasher.update(b"\x00");
        }

        if let Some(plum_relations_seal) = &plum_head.plum_relations_seal_o {
            hasher.update(b"\x01");
            hasher.update(&plum_relations_seal.value.sha256sum.value);
        } else {
            hasher.update(b"\x00");
        }

        hasher.update(&plum_head.plum_body_seal.value.sha256sum.value);

        if let Some(owner_id) = &plum_head.owner_id_o {
            hasher.update(b"\x01");
            hasher.update(&owner_id.value);
        } else {
            hasher.update(b"\x00");
        }

        if let Some(created_at) = &plum_head.created_at_o {
            hasher.update(b"\x01");
            // to_le_bytes gives little-endian representation.
            hasher.update(created_at.value.to_le_bytes());
        } else {
            hasher.update(b"\x00");
        }

        if let Some(metadata) = &plum_head.metadata_o {
            hasher.update(b"\x01");
            hasher.update(metadata);
        } else {
            hasher.update(b"\x00");
        }

        PlumHeadSeal::from(Seal::from(Sha256Sum::from(hasher.finalize().to_vec())))
    }
}
