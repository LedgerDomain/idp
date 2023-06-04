use crate::{
    did_key_from_jwk, jws_sign, OwnedData, PlumSigContent, PlumSigContentHash, Result, JWS,
};
use idp_proto::{PathState, PlumHeadSeal, PlumRelationFlags};
use std::collections::HashMap;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct PlumSig {
    /// The signature is over the sha256 hash of this PlumSigContent (see its impl of Hashable).
    pub content: PlumSigContent,
    /// The signature is over the PlumHeadSeal (which is itself a hash of the Plum).
    /// The JWS contains the DID fragment URL of the signer (e.g. `did:example:123abc#key-1`) as
    /// the "iss" field.
    pub signature: JWS,
}

impl PlumSig {
    pub async fn new(content: PlumSigContent, signer_priv_jwk: &ssi_jwk::JWK) -> Result<Self> {
        let plum_sig_content_hash = PlumSigContentHash::from(&content);
        let signature = jws_sign(signer_priv_jwk, plum_sig_content_hash.as_slice()).await?;
        assert!(signature
            .verify_against_known_signer(plum_sig_content_hash.as_slice(), signer_priv_jwk)
            .is_ok());
        Ok(Self { content, signature })
    }
    pub fn verify_against_known_signer(&self, signer_pub_jwk: &ssi_jwk::JWK) -> Result<()> {
        let plum_sig_content_hash = PlumSigContentHash::from(&self.content);
        self.signature
            .verify_against_known_signer(plum_sig_content_hash.as_slice(), signer_pub_jwk)?;
        Ok(())
    }
    pub async fn verify_and_extract_signer(&self) -> Result<ssi_dids::DIDURL> {
        let plum_sig_content_hash = PlumSigContentHash::from(&self.content);
        self.signature
            .verify_and_extract_signer(plum_sig_content_hash.as_slice())
            .await
    }
    // TODO: Potentially make the inside of this loop into its own function so that a single level
    // of verification can be done (e.g. in execute_path_state_plum_sig_create or execute_path_state_plum_sig_update)
    pub async fn verify_chain(
        plum_sig_plum_head_seal: &PlumHeadSeal,
        datahost: &mut idp_core::Datahost,
        mut datahost_transaction_o: Option<
            &mut dyn idp_datahost_storage::DatahostStorageTransaction,
        >,
    ) -> Result<()> {
        // Load the initial PlumSig and its OwnedData
        let mut plum_sig: PlumSig = datahost
            .load_plum_and_decode_and_deserialize(
                &plum_sig_plum_head_seal,
                datahost_transaction_o.as_deref_mut(),
            )
            .await?;
        let mut owned_data: OwnedData = datahost
            .load_plum_and_decode_and_deserialize(
                &plum_sig.content.plum,
                datahost_transaction_o.as_deref_mut(),
            )
            .await?;
        loop {
            // Verify the PlumSig's signature.
            let signer_did = plum_sig.verify_and_extract_signer().await?.did;
            // Verify the constraint on the previous values.
            match (
                plum_sig.content.previous_plum_sig_o.as_ref(),
                owned_data.previous_owned_data_o.as_ref(),
            ) {
                (None, None) => {
                    // No previous values, so the signer has to match the owner.
                    assert_eq!(owned_data.owner, signer_did);
                    break;
                }
                (
                    Some(previous_plum_sig_plum_head_seal),
                    Some(previous_owned_data_plum_head_seal),
                ) => {
                    let previous_plum_sig: PlumSig = datahost
                        .load_plum_and_decode_and_deserialize(
                            previous_plum_sig_plum_head_seal,
                            datahost_transaction_o.as_deref_mut(),
                        )
                        .await?;
                    anyhow::ensure!(previous_plum_sig.content.plum == *previous_owned_data_plum_head_seal, "previous PlumSig's content PlumHeadSeal did not match previous OwnedData's PlumHeadSeal");
                    let previous_owned_data: OwnedData = datahost
                        .load_plum_and_decode_and_deserialize(
                            previous_owned_data_plum_head_seal,
                            datahost_transaction_o.as_deref_mut(),
                        )
                        .await?;
                    // The previous OwnedData's owner must match the current PlumSig's signer.
                    anyhow::ensure!(
                        previous_owned_data.owner == signer_did,
                        "previous OwnedData's owner did not match current PlumSig's signer"
                    );
                    // Iterate to the previous PlumSig and OwnedData.
                    plum_sig = previous_plum_sig;
                    owned_data = previous_owned_data;
                }
                _ => {
                    anyhow::bail!("previous_plum_sig_o and previous_owned_data_o must both be either Some or both be None");
                }
            }
        }
        Ok(())
    }

    // TODO: Add params for ContentFormat and ContentEncoding
    pub async fn generate_and_store_plum_sig_owned_data_pair_without_previous(
        signer_priv_jwk: &ssi_jwk::JWK,
        data: PlumHeadSeal,
        datahost: &mut idp_core::Datahost,
        mut datahost_transaction_o: Option<
            &mut dyn idp_datahost_storage::DatahostStorageTransaction,
        >,
    ) -> Result<PlumHeadSeal> {
        let signer_did = did_key_from_jwk(&signer_priv_jwk)
            .expect("programmer error: expected signer_priv_jwk to have a key_id that is a did:key")
            .did;

        let owned_data_plum_head_seal = datahost
            .store_plum(
                &idp_proto::PlumBuilder::new()
                    .with_plum_relations_and_plum_body_content_from(
                        &OwnedData {
                            owner: signer_did.to_string(),
                            data,
                            previous_owned_data_o: None,
                        },
                        Some(&idp_proto::ContentFormat::json()),
                        idp_proto::ContentEncoding::none(),
                    )?
                    .build()?,
                datahost_transaction_o.as_deref_mut(),
            )
            .await?;
        // log::debug!(
        //     "owned_data_plum_head_seal: {}",
        //     owned_data_plum_head_seal
        // );
        let plum_sig_plum_head_seal = datahost
            .store_plum(
                &idp_proto::PlumBuilder::new()
                    .with_plum_relations_and_plum_body_content_from(
                        &PlumSig::new(
                            PlumSigContent::new(owned_data_plum_head_seal, None),
                            &signer_priv_jwk,
                        )
                        .await?,
                        Some(&idp_proto::ContentFormat::json()),
                        idp_proto::ContentEncoding::none(),
                    )?
                    .build()?,
                datahost_transaction_o.as_deref_mut(),
            )
            .await?;
        // log::debug!("plum_sig_plum_head_seal: {}", plum_sig_plum_head_seal);
        Ok(plum_sig_plum_head_seal)
    }
    // TODO: Add params for ContentFormat and ContentEncoding
    pub async fn generate_and_store_plum_sig_owned_data_pair_with_previous(
        previous_plum_sig_plum_head_seal: PlumHeadSeal,
        signer_priv_jwk: &ssi_jwk::JWK,
        owner: String,
        data: PlumHeadSeal,
        datahost: &mut idp_core::Datahost,
        mut datahost_transaction_o: Option<
            &mut dyn idp_datahost_storage::DatahostStorageTransaction,
        >,
    ) -> Result<PlumHeadSeal> {
        let signer_did = did_key_from_jwk(&signer_priv_jwk)
            .expect("programmer error: expected signer_priv_jwk to have a key_id that is a did:key")
            .did;

        // Verify that the previous PlumSig is valid.
        let previous_plum_sig: PlumSig = datahost
            .load_plum_and_decode_and_deserialize(
                &previous_plum_sig_plum_head_seal,
                datahost_transaction_o.as_deref_mut(),
            )
            .await?;
        let previous_owned_data: OwnedData = datahost
            .load_plum_and_decode_and_deserialize(
                &previous_plum_sig.content.plum,
                datahost_transaction_o.as_deref_mut(),
            )
            .await?;

        anyhow::ensure!(
            previous_owned_data.owner.as_str() == signer_did,
            "Previous OwnedData must be owned by the signer of the new PlumSig"
        );

        let owned_data_plum_head_seal = datahost
            .store_plum(
                &idp_proto::PlumBuilder::new()
                    .with_plum_relations_and_plum_body_content_from(
                        &OwnedData {
                            owner,
                            data,
                            previous_owned_data_o: Some(previous_plum_sig.content.plum),
                        },
                        Some(&idp_proto::ContentFormat::json()),
                        idp_proto::ContentEncoding::none(),
                    )?
                    .build()?,
                datahost_transaction_o.as_deref_mut(),
            )
            .await?;
        // log::debug!(
        //     "owned_data_plum_head_seal: {}",
        //     owned_data_plum_head_seal
        // );
        let plum_sig_plum_head_seal = datahost
            .store_plum(
                &idp_proto::PlumBuilder::new()
                    .with_plum_relations_and_plum_body_content_from(
                        &PlumSig::new(
                            PlumSigContent::new(
                                owned_data_plum_head_seal,
                                Some(previous_plum_sig_plum_head_seal),
                            ),
                            &signer_priv_jwk,
                        )
                        .await?,
                        Some(&idp_proto::ContentFormat::json()),
                        idp_proto::ContentEncoding::none(),
                    )?
                    .build()?,
                datahost_transaction_o.as_deref_mut(),
            )
            .await?;
        // log::debug!("plum_sig_plum_head_seal: {}", plum_sig_plum_head_seal);
        Ok(plum_sig_plum_head_seal)
    }
}

impl idp_proto::ContentClassifiable for PlumSig {
    fn content_class_str() -> &'static str {
        "application/x.idp.example.sig.PlumSig"
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

impl idp_proto::Deserializable for PlumSig {
    fn deserialize_using_format(
        content_format: &idp_proto::ContentFormat,
        reader: &mut dyn std::io::Read,
    ) -> Result<Self> {
        idp_proto::deserialize_using_serde_format(content_format, reader)
    }
}

impl idp_proto::Serializable for PlumSig {
    fn serialize_using_format(
        &self,
        content_format: &idp_proto::ContentFormat,
        writer: &mut dyn std::io::Write,
    ) -> Result<()> {
        idp_proto::serialize_using_serde_format(self, content_format, writer)
    }
}

impl idp_proto::Hashable for PlumSig {
    fn update_hasher(&self, hasher: &mut sha2::Sha256) {
        // NOTE: The specific order and form of this hashing must NOT be changed!
        self.content.update_hasher(hasher);
        self.signature.update_hasher(hasher);
    }
}

impl idp_proto::PlumRelational for PlumSig {
    fn accumulate_plum_relations_nonrecursive(
        &self,
        plum_relation_flags_m: &mut HashMap<PlumHeadSeal, PlumRelationFlags>,
    ) {
        // NOTE: Arguably this might be considered a new kind of PlumRelation "SIGNED_DEPENDENCY",
        // since this data type is simply a signature.
        match plum_relation_flags_m.get_mut(&self.content.plum) {
            Some(plum_relation_flags) => {
                *plum_relation_flags |= PlumRelationFlags::CONTENT_DEPENDENCY;
            }
            None => {
                plum_relation_flags_m.insert(
                    self.content.plum.clone(),
                    PlumRelationFlags::CONTENT_DEPENDENCY,
                );
            }
        }
    }
}

// Below are what would constitute the Governor functions for PlumSig.

pub async fn execute_path_state_plum_sig_create(
    datahost: &mut idp_core::Datahost,
    mut datahost_transaction_o: Option<&mut dyn idp_datahost_storage::DatahostStorageTransaction>,
    path: idp_proto::Path,
    plum_sig_plum_head_seal: idp_proto::PlumHeadSeal,
) -> anyhow::Result<()> {
    anyhow::ensure!(
        !datahost
            .has_path_state(&path, datahost_transaction_o.as_deref_mut())
            .await?,
        "Can't create PathState that already exists"
    );

    // TODO: Allow creating PathState with a PlumSig that has a previous PlumSig, in which case,
    // the whole history has to be checked.

    // Load up the proposed PlumSig and OwnedData and verify that the PlumSig's signer is
    // the owner of the OwnedData.  This relationship isn't necessarily true for updates,
    // since the owner can change in an update, and in that case, the signer has to match
    // the previous owner.
    let plum_sig: PlumSig = datahost
        .load_plum_and_decode_and_deserialize(
            &plum_sig_plum_head_seal,
            datahost_transaction_o.as_deref_mut(),
        )
        .await?;
    anyhow::ensure!(plum_sig.content.previous_plum_sig_o.is_none(), "Can't create a new PathState with a PlumSig that has a previous PlumSig (this may be a temporary limitation)");
    let plum_sig_signer_did = plum_sig.verify_and_extract_signer().await?.did;
    let owned_data: OwnedData = datahost
        .load_plum_and_decode_and_deserialize(
            &plum_sig.content.plum,
            datahost_transaction_o.as_deref_mut(),
        )
        .await?;
    anyhow::ensure!(owned_data.previous_owned_data_o.is_none(), "Can't create a new PathState with an OwnedData that has a previous OwnedData (this may be a temporary limitation)");
    anyhow::ensure!(
        owned_data.owner == plum_sig_signer_did,
        "Signer of new PlumSig doesn't match owner of current OwnedData"
    );
    anyhow::ensure!(
        datahost
            .has_plum(&owned_data.data, datahost_transaction_o.as_deref_mut())
            .await?,
        "OwnedData's data doesn't exist"
    );

    // Create the new PathState.
    datahost
        .insert_path_state(
            &PathState {
                path: path.clone(),
                current_state_plum_head_seal: plum_sig_plum_head_seal,
            },
            datahost_transaction_o.as_deref_mut(),
        )
        .await?;

    Ok(())
}

pub async fn execute_path_state_plum_sig_update(
    datahost: &mut idp_core::Datahost,
    mut datahost_transaction_o: Option<&mut dyn idp_datahost_storage::DatahostStorageTransaction>,
    path: idp_proto::Path,
    new_plum_sig_plum_head_seal: idp_proto::PlumHeadSeal,
) -> anyhow::Result<()> {
    // TODO: Use EnsuredTransaction.

    // Load up the current PathState and the PlumSig and OwnedData that it refers to.
    let current_path_state: PathState = datahost
        .load_path_state(&path, datahost_transaction_o.as_deref_mut())
        .await?;
    let current_plum_sig: PlumSig = datahost
        .load_plum_and_decode_and_deserialize(
            &current_path_state.current_state_plum_head_seal,
            datahost_transaction_o.as_deref_mut(),
        )
        .await?;
    let current_owned_data: OwnedData = datahost
        .load_plum_and_decode_and_deserialize(
            &current_plum_sig.content.plum,
            datahost_transaction_o.as_deref_mut(),
        )
        .await?;
    // Verify the PlumSig.
    current_plum_sig.verify_and_extract_signer().await?;

    // Load up the proposed PlumSig and OwnedData and verify that the PlumSig's signer is
    // the owner of the OwnedData.
    let new_plum_sig: PlumSig = datahost
        .load_plum_and_decode_and_deserialize(
            &new_plum_sig_plum_head_seal,
            datahost_transaction_o.as_deref_mut(),
        )
        .await?;
    let new_plum_sig_signer_did = new_plum_sig.verify_and_extract_signer().await?.did;
    let new_owned_data: OwnedData = datahost
        .load_plum_and_decode_and_deserialize(
            &new_plum_sig.content.plum,
            datahost_transaction_o.as_deref_mut(),
        )
        .await?;
    anyhow::ensure!(
        current_owned_data.owner == new_plum_sig_signer_did,
        "Signer of new PlumSig doesn't match owner of current OwnedData"
    );

    if false {
        // Also verify that if the new PlumSig has a previous PlumSig, that it is the same as the
        // current PlumSig.
        if let Some(new_plum_sig_previous_plum_sig) =
            new_plum_sig.content.previous_plum_sig_o.as_ref()
        {
            anyhow::ensure!(
                &current_path_state.current_state_plum_head_seal == new_plum_sig_previous_plum_sig,
                "New PlumSig's previous PlumSig doesn't match current PlumSig"
            );
        }
        // Also verify that if the new OwnedData has a previous OwnedData, that it is the same as the
        // current OwnedData.
        if let Some(new_owned_data_previous_owned_data) =
            new_owned_data.previous_owned_data_o.as_ref()
        {
            anyhow::ensure!(
                &current_plum_sig.content.plum == new_owned_data_previous_owned_data,
                "New OwnedData's previous OwnedData doesn't match current OwnedData"
            );
        }
        // Verify that the new PlumSig and new OwnedData either both have a previous value or both
        // don't have a previous value, so that a commutation diagram can be satisfied.
        anyhow::ensure!(
            new_plum_sig.content.previous_plum_sig_o.is_some() == new_owned_data.previous_owned_data_o.is_some(),
            "New PlumSig and new OwnedData have to both have a previous value or both not have a previous value"
        );
    } else {
        // Require that in updates, both the PlumSig and the OwnedData have previous values.
        anyhow::ensure!(
            new_plum_sig.content.previous_plum_sig_o.is_some(),
            "New PlumSig must have a previous value"
        );
        anyhow::ensure!(
            new_owned_data.previous_owned_data_o.is_some(),
            "New OwnedData must have a previous value"
        );
        anyhow::ensure!(
            &current_path_state.current_state_plum_head_seal
                == new_plum_sig.content.previous_plum_sig_o.as_ref().unwrap(),
            "New PlumSig's previous PlumSig doesn't match current PlumSig"
        );
        anyhow::ensure!(
            &current_plum_sig.content.plum
                == new_owned_data.previous_owned_data_o.as_ref().unwrap(),
            "New OwnedData's previous OwnedData doesn't match current OwnedData"
        );
    }

    // At this point a commutative diagram is satisfied:
    //
    //      current PlumSig <-prev-- new PlumSig
    //
    //             |                      |
    //           signs                  signs
    //             V                      V
    //
    //    current OwnedData <-prev-- new OwnedData
    //
    // And the signer of new PlumSig is equal to the owner of OwnedData,
    // which could be represented by a diagonal line.

    // NOTE: There might be kinds of Plum-s under OwnedData which need to satisfy a commutative
    // diagram, but they should be handled by a separate governor.

    // We can now update the PathState to point to the new PlumSig.
    let new_path_state = PathState {
        path,
        current_state_plum_head_seal: new_plum_sig_plum_head_seal,
    };
    datahost
        .update_path_state(&new_path_state, datahost_transaction_o.as_deref_mut())
        .await?;

    Ok(())
}
