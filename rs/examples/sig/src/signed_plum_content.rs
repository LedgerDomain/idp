use idp_proto::{Nonce, PlumHeadSeal, UnixNanoseconds};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct SignedPlumContent {
    pub nonce: Nonce,
    pub signed_at: UnixNanoseconds,
    pub signed_plum: PlumHeadSeal,
}

impl idp_proto::Hashable for SignedPlumContent {
    fn update_hasher(&self, hasher: &mut sha2::Sha256) {
        // NOTE: The specific order and form of this hashing must NOT be changed!
        self.nonce.update_hasher(hasher);
        self.signed_at.update_hasher(hasher);
        self.signed_plum.update_hasher(hasher);
    }
}
