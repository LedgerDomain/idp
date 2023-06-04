use idp_proto::{Nonce, PlumHeadSeal};

// TODO: Consider renaming this to PlumSigEnvelope.
#[derive(serde::Deserialize, serde::Serialize)]
pub struct PlumSigContent {
    /// Mandatory nonce to prevent known-plaintext attacks.
    pub nonce: Nonce,
    /// This is the Plum that is being signed.
    pub plum: PlumHeadSeal,
    /// Can optionally be used to turn the PlumSig into a node in a microledger of PlumSig-s.
    pub previous_plum_sig_o: Option<PlumHeadSeal>,
}

impl PlumSigContent {
    /// Construct a PlumSigContent with a randomly generated Nonce.
    pub fn new(plum: PlumHeadSeal, previous_plum_sig_o: Option<PlumHeadSeal>) -> Self {
        Self {
            nonce: Nonce::generate(),
            plum,
            previous_plum_sig_o,
        }
    }
    /// Construct a PlumSigContent with a given Nonce.
    pub fn new_with_nonce(
        nonce: Nonce,
        plum: PlumHeadSeal,
        previous_plum_sig_o: Option<PlumHeadSeal>,
    ) -> Self {
        Self {
            nonce,
            plum,
            previous_plum_sig_o,
        }
    }
}

impl idp_proto::Hashable for PlumSigContent {
    fn update_hasher(&self, hasher: &mut sha2::Sha256) {
        // NOTE: The specific order and form of this hashing must NOT be changed!
        self.nonce.update_hasher(hasher);
        self.plum.update_hasher(hasher);
        self.previous_plum_sig_o.update_hasher(hasher);
    }
}
