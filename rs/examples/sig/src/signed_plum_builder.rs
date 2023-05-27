use crate::{jws_sign, Result, SignedPlum, SignedPlumContent, SignedPlumContentHash};
use idp_proto::{Nonce, PlumHeadSeal, UnixNanoseconds};

/// If with_nonce isn't used, then a random nonce will be generated upon build.  If with_signed_at
/// isn't used, then "now" will be upon build.  Using with_signed_plum is mandatory.
#[derive(Default)]
pub struct SignedPlumBuilder {
    pub nonce_o: Option<Nonce>,
    pub signed_at_o: Option<UnixNanoseconds>,
    pub signed_plum_o: Option<PlumHeadSeal>,
}

impl SignedPlumBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    /// Build the SignedPlum and sign it using the given JWK.
    pub async fn build_and_sign(self, priv_jwk: &ssi_jwk::JWK) -> Result<SignedPlum> {
        anyhow::ensure!(
            self.signed_plum_o.is_some(),
            "must use with_signed_plum to set the PlumHeadSeal of the plum to be signed"
        );
        let nonce = self.nonce_o.or_else(|| Some(Nonce::generate())).unwrap();
        let signed_at = self
            .signed_at_o
            .or_else(|| Some(UnixNanoseconds::now()))
            .unwrap();
        let signed_plum = self.signed_plum_o.unwrap();

        let signed_plum_content = SignedPlumContent {
            nonce,
            signed_at,
            signed_plum,
        };
        let signed_plum_content_hash = SignedPlumContentHash::from(&signed_plum_content);
        let signature = jws_sign(priv_jwk, signed_plum_content_hash.as_slice()).await?;

        Ok(SignedPlum {
            content: signed_plum_content,
            signature,
        })
    }
    pub fn with_nonce(mut self, nonce: Nonce) -> Self {
        self.nonce_o = Some(nonce);
        self
    }
    pub fn with_signed_at(mut self, signed_at: UnixNanoseconds) -> Self {
        self.signed_at_o = Some(signed_at);
        self
    }
    pub fn with_signed_plum(mut self, signed_plum: PlumHeadSeal) -> Self {
        self.signed_plum_o = Some(signed_plum);
        self
    }
}
