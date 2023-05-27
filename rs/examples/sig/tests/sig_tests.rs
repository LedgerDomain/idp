use idp_proto::{ContentEncoding, PlumBuilder, PlumHeadSeal};
use sig::{did_key_from_jwk, KeyType, SignedPlumBuilder};

/// This will run once at load time (i.e. presumably before main function is called).
#[ctor::ctor]
fn overall_init() {
    env_logger::init();
}

#[tokio::test]
async fn test_sig() {
    // Generate a private key for signing.
    let signer_priv_jwk = KeyType::Secp256k1.generate_priv_jwk().expect("pass");
    let signer_pub_jwk = signer_priv_jwk.to_public();
    let signer_did = did_key_from_jwk(&signer_pub_jwk).expect("pass");
    log::debug!("signer_did: {:?}", signer_did);

    // Make a Plum that will be signed.
    let content_1 = "splunges are cool".to_string();
    let content_1_plum = PlumBuilder::new()
        .with_plum_relations_and_plum_body_content_from(&content_1, None, ContentEncoding::none())
        .expect("pass")
        .build()
        .expect("pass");
    let content_1_plum_head_seal = PlumHeadSeal::from(&content_1_plum);

    let signed_plum = SignedPlumBuilder::new()
        .with_signed_plum(content_1_plum_head_seal)
        .build_and_sign(&signer_priv_jwk)
        .await
        .expect("pass");
    signed_plum.verify_against_known_signer(&signer_pub_jwk).expect("pass");
    let extracted_signer_did = signed_plum.verify_and_extract_signer().await.expect("pass");
    log::debug!("extracted_signer_did: {:?}", extracted_signer_did);
    assert_eq!(extracted_signer_did.did, signer_did.did);
}
