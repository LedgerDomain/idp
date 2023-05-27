use crate::{did_key_from_jwk, with_multibase_fragment, Result};

#[derive(Clone, Copy, Debug)]
pub enum KeyType {
    Secp256k1,
}

impl KeyType {
    /// Generate a priv JWK of the specified type and set its key_id field to the corresponding did:key value.
    pub fn generate_priv_jwk(&self) -> Result<ssi_jwk::JWK> {
        let mut priv_jwk = match self {
            Self::Secp256k1 => ssi_jwk::JWK::generate_secp256k1()?,
        };

        let did_key = did_key_from_jwk(&priv_jwk).unwrap();
        let did_fragment_url =
            with_multibase_fragment(did_key.clone().into(), &did_key.to_string());

        priv_jwk.key_id = Some(did_fragment_url.to_string());
        Ok(priv_jwk)
    }
}
