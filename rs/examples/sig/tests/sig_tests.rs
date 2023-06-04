use async_lock::RwLock;
use std::sync::Arc;

/// This will run once at load time (i.e. presumably before main function is called).
#[ctor::ctor]
fn overall_init() {
    env_logger::init();
}

#[tokio::test]
async fn test_sig() {
    // Generate a private key for signing.
    let signer_priv_jwk = sig::KeyType::Secp256k1.generate_priv_jwk().expect("pass");
    let signer_pub_jwk = signer_priv_jwk.to_public();
    let signer_did = sig::did_key_from_jwk(&signer_pub_jwk).expect("pass");
    log::debug!("signer_did: {:?}", signer_did);

    // Make a Plum that will be signed.
    let content_0 = "splunges are cool".to_string();
    let content_0_plum = idp_proto::PlumBuilder::new()
        .with_plum_relations_and_plum_body_content_from(
            &content_0,
            None,
            idp_proto::ContentEncoding::none(),
        )
        .expect("pass")
        .build()
        .expect("pass");
    let content_0_plum_head_seal = idp_proto::PlumHeadSeal::from(&content_0_plum);

    let plum_sig = sig::PlumSig::new(
        sig::PlumSigContent::new(content_0_plum_head_seal, None),
        &signer_priv_jwk,
    )
    .await
    .expect("pass");
    plum_sig
        .verify_against_known_signer(&signer_pub_jwk)
        .expect("pass");
    let extracted_signer_did = plum_sig.verify_and_extract_signer().await.expect("pass");
    log::debug!("extracted_signer_did: {:?}", extracted_signer_did);
    assert_eq!(extracted_signer_did.did, signer_did.did);
}

#[tokio::test]
async fn test_plum_sig() {
    // Generate 2 private keys for signing.  Each one represents a different owner.
    let signer_0_priv_jwk = sig::KeyType::Secp256k1.generate_priv_jwk().expect("pass");
    let signer_0_pub_jwk = signer_0_priv_jwk.to_public();
    let signer_0_did = sig::did_key_from_jwk(&signer_0_pub_jwk).expect("pass").did;
    log::debug!("signer_0_did: {:?}", signer_0_did);
    let signer_1_priv_jwk = sig::KeyType::Secp256k1.generate_priv_jwk().expect("pass");
    let signer_1_pub_jwk = signer_1_priv_jwk.to_public();
    let signer_1_did = sig::did_key_from_jwk(&signer_1_pub_jwk).expect("pass").did;
    log::debug!("signer_1_did: {:?}", signer_1_did);

    let path = idp_proto::Path::from(format!("test_path_for_plum_sig_{}", uuid::Uuid::new_v4()));

    // Create the Datahost that will store the Plum-s and PathState-s.
    let datahost_la = {
        // Regarding `?mode=rwc`, see https://github.com/launchbadge/sqlx/issues/1114#issuecomment-827815038
        let database_url = "sqlite:sig_tests.db?mode=rwc";
        let datahost = idp_core::Datahost::open(
            idp_datahost_storage_sqlite::DatahostStorageSQLite::connect_and_run_migrations(
                database_url,
            )
            .await
            .expect("pass"),
        );
        Arc::new(RwLock::new(datahost))
    };
    let mut datahost_g = datahost_la.write().await;

    // Create a bunch of content Plum-s.  Note that instead of a loop, one could use
    // futures::future::try_join_all (see https://stackoverflow.com/questions/68344087/how-do-you-call-an-async-method-within-a-closure-like-within-map-in-rust),
    // and that would run all the async calls in parallel.
    let mut content_plum_head_seal_v = Vec::new();
    for content_str in [
        "ostriches run all funky",
        "donkeys run all regular",
        "now *I* am the owner!",
        "and *I* declare that humans rule!",
    ]
    .into_iter()
    {
        let content_plum_head_seal = datahost_g
            .store_plum(
                &idp_proto::PlumBuilder::new()
                    .with_plum_relations_and_plum_body_content_from(
                        &content_str.to_string(),
                        None,
                        idp_proto::ContentEncoding::none(),
                    )
                    .expect("pass")
                    .build()
                    .expect("pass"),
                None,
            )
            .await
            .expect("pass");
        content_plum_head_seal_v.push(content_plum_head_seal);
    }

    // Must use without_previous for the first PlumSig in a chain.
    let plum_sig_0_plum_head_seal =
        sig::PlumSig::generate_and_store_plum_sig_owned_data_pair_without_previous(
            &signer_0_priv_jwk,
            content_plum_head_seal_v[0].clone(),
            &mut datahost_g,
            None,
        )
        .await
        .expect("pass");

    // Create the PathState.
    sig::execute_path_state_plum_sig_create(
        &mut datahost_g,
        None,
        path.clone(),
        plum_sig_0_plum_head_seal.clone(),
    )
    .await
    .expect("pass");
    // Verify it.
    assert_eq!(
        datahost_g.load_path_state(&path, None).await.expect("pass"),
        idp_proto::PathState {
            path: path.clone(),
            owner_o: None,
            current_state_plum_head_seal: plum_sig_0_plum_head_seal.clone()
        }
    );
    sig::PlumSig::verify_chain(&plum_sig_0_plum_head_seal, &mut datahost_g, None)
        .await
        .expect("pass");

    let plum_sig_1_plum_head_seal =
        sig::PlumSig::generate_and_store_plum_sig_owned_data_pair_with_previous(
            plum_sig_0_plum_head_seal,
            &signer_0_priv_jwk,
            signer_0_did.clone(),
            content_plum_head_seal_v[1].clone(),
            &mut datahost_g,
            None,
        )
        .await
        .expect("pass");

    // Update the PathState.
    sig::execute_path_state_plum_sig_update(
        &mut datahost_g,
        None,
        path.clone(),
        plum_sig_1_plum_head_seal.clone(),
    )
    .await
    .expect("pass");
    // Verify it.
    assert_eq!(
        datahost_g.load_path_state(&path, None).await.expect("pass"),
        idp_proto::PathState {
            path: path.clone(),
            owner_o: None,
            current_state_plum_head_seal: plum_sig_1_plum_head_seal.clone()
        }
    );
    sig::PlumSig::verify_chain(&plum_sig_1_plum_head_seal, &mut datahost_g, None)
        .await
        .expect("pass");

    let plum_sig_2_plum_head_seal =
        sig::PlumSig::generate_and_store_plum_sig_owned_data_pair_with_previous(
            plum_sig_1_plum_head_seal,
            &signer_0_priv_jwk,
            // NOTE that the signer changed from signer_0_did to signer_1_did.
            signer_1_did.clone(),
            content_plum_head_seal_v[2].clone(),
            &mut datahost_g,
            None,
        )
        .await
        .expect("pass");

    // Update the PathState.
    sig::execute_path_state_plum_sig_update(
        &mut datahost_g,
        None,
        path.clone(),
        plum_sig_2_plum_head_seal.clone(),
    )
    .await
    .expect("pass");
    // Verify it
    assert_eq!(
        datahost_g.load_path_state(&path, None).await.expect("pass"),
        idp_proto::PathState {
            path: path.clone(),
            owner_o: None,
            current_state_plum_head_seal: plum_sig_2_plum_head_seal.clone()
        }
    );
    sig::PlumSig::verify_chain(&plum_sig_2_plum_head_seal, &mut datahost_g, None)
        .await
        .expect("pass");

    let plum_sig_3_plum_head_seal =
        sig::PlumSig::generate_and_store_plum_sig_owned_data_pair_with_previous(
            plum_sig_2_plum_head_seal,
            // NOTE that the signer is now signer_1, which must match the previous OwnedData's owner.
            &signer_1_priv_jwk,
            signer_1_did.clone(),
            content_plum_head_seal_v[3].clone(),
            &mut datahost_g,
            None,
        )
        .await
        .expect("pass");

    // Update the PathState.
    sig::execute_path_state_plum_sig_update(
        &mut datahost_g,
        None,
        path.clone(),
        plum_sig_3_plum_head_seal.clone(),
    )
    .await
    .expect("pass");
    // Verify it.
    assert_eq!(
        datahost_g.load_path_state(&path, None).await.expect("pass"),
        idp_proto::PathState {
            path: path.clone(),
            owner_o: None,
            current_state_plum_head_seal: plum_sig_3_plum_head_seal.clone()
        }
    );
    sig::PlumSig::verify_chain(&plum_sig_3_plum_head_seal, &mut datahost_g, None)
        .await
        .expect("pass");
}
