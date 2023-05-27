use crate::SignedPlumContent;

#[derive(
    Clone,
    derive_more::Deref,
    serde::Deserialize,
    Eq,
    derive_more::From,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    serde::Serialize,
)]
pub struct SignedPlumContentHash(Vec<u8>);

impl From<&SignedPlumContent> for SignedPlumContentHash {
    fn from(signed_plum_content: &SignedPlumContent) -> Self {
        use sha2::Digest;

        // For now, a seal is only the Sha256Sum, but it could be other stuff later.
        let mut hasher = sha2::Sha256::new();

        use idp_proto::Hashable;
        signed_plum_content.update_hasher(&mut hasher);

        Self(hasher.finalize().to_vec())
    }
}
