use crate::PlumSigContent;

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
pub struct PlumSigContentHash(Vec<u8>);

impl From<&PlumSigContent> for PlumSigContentHash {
    fn from(plum_sig_content: &PlumSigContent) -> Self {
        use sha2::Digest;

        // For now, a seal is only the Sha256Sum, but it could be other stuff later.
        let mut hasher = sha2::Sha256::new();

        use idp_proto::Hashable;
        plum_sig_content.update_hasher(&mut hasher);

        Self(hasher.finalize().to_vec())
    }
}
