use crate::{PlumBody, PlumBodySeal, Seal, Sha256Sum};

impl AsRef<[u8]> for PlumBodySeal {
    fn as_ref(&self) -> &[u8] {
        self.value.as_ref()
    }
}

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

        // NOTE: The specific order and form of this hashing must NOT be changed!

        if let Some(plum_body_nonce) = &plum_body.plum_body_nonce_o {
            hasher.update(b"\x01");
            hasher.update(&plum_body_nonce.value);
        } else {
            hasher.update(b"\x00");
        }

        // to_le_bytes gives little-endian representation.
        hasher.update(plum_body.plum_body_content_length.to_le_bytes());

        hasher.update(&plum_body.plum_body_content_type.value);

        hasher.update(&plum_body.plum_body_content);

        PlumBodySeal::from(Seal::from(Sha256Sum::from(hasher.finalize().to_vec())))
    }
}
