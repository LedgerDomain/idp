use crate::{jws_sign, JWS, Result, SignedPlumContent, SignedPlumContentHash};
use idp_proto::{PlumHeadSeal, PlumRelationFlags};
use std::collections::HashMap;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct SignedPlum {
    /// The signature is over the sha256 hash of this SignedPlumContent (see its impl of Hashable).
    pub content: SignedPlumContent,
    /// The signature is over the PlumHeadSeal (which is itself a hash of the Plum).
    /// The JWS contains the DID fragment URL of the signer (e.g. `did:example:123abc#key-1`) as the "iss" field.
    pub signature: JWS,
}

impl SignedPlum {
    pub async fn new(content: SignedPlumContent, signer_priv_jwk: &ssi_jwk::JWK) -> Result<Self> {
        let signed_plum_content_hash = SignedPlumContentHash::from(&content);
        let signature = jws_sign(signer_priv_jwk, signed_plum_content_hash.as_slice()).await?;
        assert!(signature
            .verify_against_known_signer(signed_plum_content_hash.as_slice(), signer_priv_jwk)
            .is_ok());
        Ok(Self { content, signature })
    }
    pub fn verify_against_known_signer(&self, signer_pub_jwk: &ssi_jwk::JWK) -> Result<()> {
        let signed_plum_content_hash = SignedPlumContentHash::from(&self.content);
        self.signature
            .verify_against_known_signer(signed_plum_content_hash.as_slice(), signer_pub_jwk)?;
        Ok(())
    }
    pub async fn verify_and_extract_signer(&self) -> Result<ssi_dids::DIDURL> {
        let signed_plum_content_hash = SignedPlumContentHash::from(&self.content);
        self.signature
            .verify_and_extract_signer(signed_plum_content_hash.as_slice())
            .await
    }
}

impl idp_proto::ContentClassifiable for SignedPlum {
    fn content_class_str() -> &'static str {
        "application/x.idp.example.sig.SignedPlum"
    }
    fn derive_content_class_str(&self) -> &'static str {
        Self::content_class_str()
    }
    fn default_content_format(&self) -> Option<idp_proto::ContentFormat> {
        None
    }
    fn validate_content_format(&self, content_format: &idp_proto::ContentFormat) -> Result<()> {
        idp_proto::validate_is_serde_format(content_format)
    }
}

impl idp_proto::Deserializable for SignedPlum {
    fn deserialize_using_format(
        content_format: &idp_proto::ContentFormat,
        reader: &mut dyn std::io::Read,
    ) -> Result<Self> {
        idp_proto::deserialize_using_serde_format(content_format, reader)
    }
}

impl idp_proto::Serializable for SignedPlum {
    fn serialize_using_format(
        &self,
        content_format: &idp_proto::ContentFormat,
        writer: &mut dyn std::io::Write,
    ) -> Result<()> {
        idp_proto::serialize_using_serde_format(self, content_format, writer)
    }
}

impl idp_proto::PlumRelational for SignedPlum {
    fn accumulate_plum_relations_nonrecursive(
        &self,
        plum_relation_flags_m: &mut HashMap<PlumHeadSeal, PlumRelationFlags>,
    ) {
        // NOTE: Arguably this might be considered a new kind of PlumRelation "SIGNED_DEPENDENCY",
        // since this data type is simply a signature.
        match plum_relation_flags_m.get_mut(&self.content.signed_plum) {
            Some(plum_relation_flags) => {
                *plum_relation_flags |= PlumRelationFlags::CONTENT_DEPENDENCY;
            }
            None => {
                plum_relation_flags_m
                    .insert(self.content.signed_plum.clone(), PlumRelationFlags::CONTENT_DEPENDENCY);
            }
        }
    }
}
