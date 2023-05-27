use crate::Result;
use anyhow::Context;

/// Sign a message, producing a JWS.
pub async fn jws_sign(priv_jwk: &ssi_jwk::JWK, message: &[u8]) -> Result<JWS> {
    anyhow::ensure!(
        priv_jwk.key_id.is_some(),
        "JWK is missing key_id attribute; can't properly sign."
    );
    let jws = JWS::from(ssi_jws::detached_sign_unencoded_payload(
        priv_jwk
            .get_algorithm()
            .ok_or_else(|| anyhow::anyhow!("could not determine algorithm from JWK"))?,
        message,
        priv_jwk,
    )?);
    Ok(jws)
}

/// A JSON Web Signature value.  See <https://en.wikipedia.org/wiki/JSON_Web_Signature>
#[derive(
    Clone,
    Debug,
    derive_more::Deref,
    serde::Deserialize,
    derive_more::Display,
    Eq,
    derive_more::From,
    derive_more::Into,
    PartialEq,
    serde::Serialize,
)]
pub struct JWS(String);

impl JWS {
    fn extract_header(&self) -> Result<ssi_jws::Header> {
        let (header_b64, _) = ssi_jws::split_detached_jws(self.as_str())?;
        let header_json = base64::decode_config(header_b64, base64::URL_SAFE_NO_PAD)?;
        let header: ssi_jws::Header = serde_json::from_slice(&header_json)?;
        Ok(header)
    }
    pub fn extract_signer_did_fragment_url(&self) -> Result<ssi_dids::DIDURL> {
        let header = self.extract_header()?;
        log::trace!(
            "JWS::extract_signer_did_fragment_url: header = {:?}",
            header
        );
        anyhow::ensure!(header.key_id.is_some(), "missing 'kid' attribute");
        let key_id = header.key_id.unwrap();
        let signer_did_fragment_url = ssi_dids::DIDURL::try_from(key_id)
            .context("JWS 'kid' attribute was not a DID fragment URL")?;
        anyhow::ensure!(
            signer_did_fragment_url.fragment.is_some(),
            "JWS 'kid' attribute was not a DID fragment URL"
        );
        Ok(signer_did_fragment_url)
    }
    /// This will verify this JWS against a specified pub key, returning error if the verification
    /// failed.  In particular, if the JWS's header's key ID doesn't match the one in signer_pub_jwk,
    /// then this call will fail with error.  No DID resolution will be done, so note that the
    /// caller is responsible for ensuring that the key_id within signer_pub_jwk is correct.
    pub fn verify_against_known_signer(
        &self,
        payload: &[u8],
        signer_pub_jwk: &ssi_jwk::JWK,
    ) -> Result<()> {
        // Retrieve the signer's DID fragment URL from the JWS.
        let jws_signer_did_fragment_url = self.extract_signer_did_fragment_url()?;
        // Verify the signature.  This will return with error here if verification fails.
        ssi_jws::detached_verify(self.as_str(), payload, signer_pub_jwk)?;
        // TODO: Maybe tolerate a key_id that's None.
        anyhow::ensure!(signer_pub_jwk.key_id == Some(jws_signer_did_fragment_url.to_string()), "signature verified but there was a key_id mismatch between signer_pub_jwk and the JWS header");

        Ok(())
    }

    /// This will verify this JWS and return the DIDFragmentURL (i.e. primary DID and the fragment
    /// specifying the key) of the signer.
    pub async fn verify_and_extract_signer(&self, payload: &[u8]) -> Result<ssi_dids::DIDURL> {
        // Retrieve the signer's DID fragment URL from the JWS.
        let signer_did_fragment_url = self.extract_signer_did_fragment_url()?;
        // Retrieve the JWK from the key ID.  This involves a DID resolution.
        let signer_pub_jwk = resolve_to_pub_jwk(&signer_did_fragment_url).await?;
        // Verify the signature.  This will return with error here if verification fails.
        ssi_jws::detached_verify(self.as_str(), payload, &signer_pub_jwk)?;

        Ok(signer_did_fragment_url)
    }
}

/// This will resolve this DIDFragmentURL and extract the specified pub key.
async fn resolve_to_pub_jwk(did_fragment_url: &ssi_dids::DIDURL) -> Result<ssi_jwk::JWK> {
    anyhow::ensure!(
        did_fragment_url.fragment.is_some(),
        "DID fragment URL is missing fragment"
    );

    // Retrieve the JWK from the key ID.  This involves a DID resolution.
    let (_resolution_metadata, did_document_o, _did_document_metadata_o) = crate::did_resolver()
        .resolve(&did_fragment_url.did, &Default::default())
        .await;
    anyhow::ensure!(
        did_document_o.is_some(),
        "DID resolution failed for {}",
        did_fragment_url.did
    );

    let did_document = did_document_o.unwrap();
    let selected_did_document_object =
        did_document
            .select_object(did_fragment_url)
            .with_context(|| {
                format!(
                    "failed to select object {} from DID document for {}",
                    did_fragment_url, did_fragment_url.did
                )
            })?;
    let pub_jwk = match selected_did_document_object {
        ssi_dids::Resource::VerificationMethod(verification_method_map) => {
            // TODO: Need to ensure this verification method is AssertionMethod (which is not possible
            // simply through ssi::did::Document::select_object).  For now, just allow any key purpose.
            verification_method_map.get_jwk()?
        }
        _ => {
            anyhow::bail!(
                "expected {} to resolve to a verification method",
                did_fragment_url
            );
        }
    };
    Ok(pub_jwk)
}
