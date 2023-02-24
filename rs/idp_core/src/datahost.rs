use crate::{
    BranchNode, DatahostStorage, DatahostStorageError, DatahostStorageTransaction, DirNode,
    FragmentQueryResult, FragmentQueryable,
};
use anyhow::Result;
use idp_proto::{
    Path, PathState, Plum, PlumBody, PlumBodySeal, PlumHead, PlumHeadSeal, PlumRelationFlags,
    PlumRelations, PlumRelationsSeal,
};
use std::{collections::HashMap, convert::TryFrom};

pub struct Datahost {
    datahost_storage_b: Box<dyn DatahostStorage>,
}

// TEMP HACK (maybe?)
unsafe impl Send for Datahost {}
// TEMP HACK (maybe?)
unsafe impl Sync for Datahost {}

impl Datahost {
    pub fn open(datahost_storage: impl DatahostStorage + 'static) -> Self {
        Self {
            datahost_storage_b: Box::new(datahost_storage),
        }
    }

    //
    // Data methods
    //

    pub async fn has_plum_head(
        &self,
        plum_head_seal: &PlumHeadSeal,
    ) -> Result<bool, DatahostStorageError> {
        let mut transaction_b = self.datahost_storage_b.begin_transaction().await?;
        let has_plum_head = self
            .datahost_storage_b
            .has_plum_head(transaction_b.as_mut(), plum_head_seal)
            .await?;
        transaction_b.commit().await?;
        Ok(has_plum_head)
    }
    pub async fn has_plum_relations(
        &self,
        plum_relations_seal: &PlumRelationsSeal,
    ) -> Result<bool, DatahostStorageError> {
        let mut transaction_b = self.datahost_storage_b.begin_transaction().await?;
        let has_plum_relations = self
            .datahost_storage_b
            .has_plum_relations(transaction_b.as_mut(), plum_relations_seal)
            .await?;
        transaction_b.commit().await?;
        Ok(has_plum_relations)
    }
    pub async fn has_plum_body(
        &self,
        plum_body_seal: &PlumBodySeal,
    ) -> Result<bool, DatahostStorageError> {
        let mut transaction_b = self.datahost_storage_b.begin_transaction().await?;
        let has_plum_body = self
            .datahost_storage_b
            .has_plum_body(transaction_b.as_mut(), plum_body_seal)
            .await?;
        transaction_b.commit().await?;
        Ok(has_plum_body)
    }
    pub async fn has_plum(
        &self,
        plum_head_seal: &PlumHeadSeal,
    ) -> Result<bool, DatahostStorageError> {
        let mut transaction_b = self.datahost_storage_b.begin_transaction().await?;
        let has_plum = self
            .datahost_storage_b
            .has_plum(transaction_b.as_mut(), plum_head_seal)
            .await?;
        transaction_b.commit().await?;
        Ok(has_plum)
    }
    pub async fn store_plum_head(&self, plum_head: &PlumHead) -> Result<PlumHeadSeal> {
        log::trace!(
            "Datahost::store_plum_head; PlumHeadSeal is {}",
            PlumHeadSeal::from(plum_head)
        );

        let mut transaction_b = self.datahost_storage_b.begin_transaction().await?;
        let plum_head_seal = self
            .datahost_storage_b
            .store_plum_head(transaction_b.as_mut(), plum_head)
            .await?;
        transaction_b.commit().await?;
        Ok(plum_head_seal)
    }

    pub async fn store_plum_relations(
        &self,
        plum_relations: &PlumRelations,
    ) -> Result<PlumRelationsSeal> {
        log::trace!(
            "Datahost::store_plum_relations; PlumRelationsSeal is {}",
            PlumRelationsSeal::from(plum_relations)
        );

        let mut transaction_b = self.datahost_storage_b.begin_transaction().await?;
        let plum_relations_seal = self
            .datahost_storage_b
            .store_plum_relations(transaction_b.as_mut(), plum_relations)
            .await?;
        transaction_b.commit().await?;
        Ok(plum_relations_seal)
    }

    pub async fn store_plum_body(&self, plum_body: &PlumBody) -> Result<PlumBodySeal> {
        log::trace!(
            "Datahost::store_plum_body; PlumBodySeal is {}",
            PlumBodySeal::from(plum_body)
        );

        let mut transaction_b = self.datahost_storage_b.begin_transaction().await?;
        let plum_body_seal = self
            .datahost_storage_b
            .store_plum_body(transaction_b.as_mut(), plum_body)
            .await?;
        transaction_b.commit().await?;
        Ok(plum_body_seal)
    }

    pub async fn store_plum(&self, plum: &Plum) -> Result<PlumHeadSeal> {
        log::debug!(
            "Datahost::store_plum; plum's PlumHeadSeal is {}",
            PlumHeadSeal::from(&plum.plum_head),
        );
        let mut transaction_b = self.datahost_storage_b.begin_transaction().await?;
        let plum_head_seal = self
            .datahost_storage_b
            .store_plum(transaction_b.as_mut(), plum)
            .await?;
        transaction_b.commit().await?;
        Ok(plum_head_seal)
    }

    /// If the specified PlumHead doesn't exist in this Datahost, returns error.
    pub async fn load_plum_head(&self, plum_head_seal: &PlumHeadSeal) -> Result<PlumHead> {
        let mut transaction_b = self.datahost_storage_b.begin_transaction().await?;
        let plum_head = self
            .datahost_storage_b
            .load_plum_head(transaction_b.as_mut(), plum_head_seal)
            .await?;
        transaction_b.commit().await?;
        Ok(plum_head)
    }

    /// If either of the PlumHead or PlumBody for the specified Plum doesn't exist in this Datahost,
    /// returns None.
    pub async fn load_option_plum(&self, plum_head_seal: &PlumHeadSeal) -> Result<Option<Plum>> {
        let mut transaction_b = self.datahost_storage_b.begin_transaction().await?;
        let plum_o = self
            .datahost_storage_b
            .load_option_plum(transaction_b.as_mut(), plum_head_seal)
            .await?;
        transaction_b.commit().await?;
        Ok(plum_o)
    }
    pub async fn load_plum(&self, plum_head_seal: &PlumHeadSeal) -> Result<Plum> {
        let mut transaction_b = self.datahost_storage_b.begin_transaction().await?;
        let plum = self
            .datahost_storage_b
            .load_plum(transaction_b.as_mut(), plum_head_seal)
            .await?;
        transaction_b.commit().await?;
        Ok(plum)
    }

    //
    // Methods for determining plum_relations between Plums
    //

    pub async fn accumulated_relations_recursive(
        &self,
        plum_head_seal: &PlumHeadSeal,
        mask: PlumRelationFlags,
    ) -> Result<HashMap<PlumHeadSeal, PlumRelationFlags>> {
        let mut transaction_b = self.datahost_storage_b.begin_transaction().await?;
        let mut plum_relation_flags_m = HashMap::new();
        self.accumulate_relations_recursive_impl(
            transaction_b.as_mut(),
            plum_head_seal,
            mask,
            &mut plum_relation_flags_m,
        )
        .await?;
        transaction_b.commit().await?;
        Ok(plum_relation_flags_m)
    }

    /// The reason this is Pin<Box<dyn Future<Output = ...>>> is to allow recursive async calls.
    /// See `rustc --explain E0733` for an explanation.  The explicit lifetimes are needed, see:
    /// https://stackoverflow.com/questions/59538812/why-recursive-async-functions-require-static-parameters-in-rust
    fn accumulate_relations_recursive_impl<'a>(
        &'a self,
        transaction: &'a mut dyn DatahostStorageTransaction,
        plum_head_seal: &'a PlumHeadSeal,
        mask: PlumRelationFlags,
        plum_relation_flags_m: &'a mut HashMap<PlumHeadSeal, PlumRelationFlags>,
        // TODO: Need some way of indicating which Plums didn't have plum_relations present in the Datahost,
        // so that the client can act appropriately.
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            // Note that this assumes there are no cycles, which is true by construction because relations
            // are defined using the PlumHeadSeal of the target Plum, and therefore it would be infeasible
            // to construct a cycle.

            if plum_relation_flags_m.contains_key(plum_head_seal) {
                // Already traversed; nothing to do.
                return Ok(());
            }

            // Recurse on the plum_relations for this plum_head_seal.
            let inner_relation_flags_m = {
                let mut inner_relation_flags_m: HashMap<PlumHeadSeal, PlumRelationFlags> =
                    HashMap::new();

                // TODO: This could be more efficient with more-specific DatahostStorage methods.
                let plum_head = self
                    .datahost_storage_b
                    .load_plum_head(&mut *transaction, plum_head_seal)
                    .await?;
                if plum_head.plum_relations_seal_o.is_none() {
                    // No relations, so there's nothing to traverse to.
                    return Ok(());
                }
                let plum_relations_seal = plum_head.plum_relations_seal_o.unwrap();
                let plum_relations = self
                    .datahost_storage_b
                    .load_plum_relations(&mut *transaction, &plum_relations_seal)
                    .await?;

                for plum_relation_flags_mapping in plum_relations.plum_relation_flags_mapping_v {
                    log::trace!(
                        "accumulate_relations_recursive_impl; {} -> {}",
                        plum_head_seal,
                        plum_relation_flags_mapping.target_plum_head_seal
                    );
                    let plum_relation_flags = PlumRelationFlags::try_from(
                        plum_relation_flags_mapping.plum_relation_flags_raw,
                    )?;
                    let masked_relation_flags = mask & plum_relation_flags;
                    // Only do anything if the masked flags are nonzero.
                    if masked_relation_flags != PlumRelationFlags::NONE {
                        match inner_relation_flags_m
                            .get_mut(&plum_relation_flags_mapping.target_plum_head_seal)
                        {
                            Some(inner_relation_flags) => {
                                *inner_relation_flags |= masked_relation_flags;
                            }
                            None => {
                                inner_relation_flags_m.insert(
                                    plum_relation_flags_mapping.target_plum_head_seal.clone(),
                                    masked_relation_flags,
                                );
                            }
                        }
                    }
                }
                inner_relation_flags_m
            };

            // Now go through the accumulated inner_relation_flags_m and recurse.
            for (inner_plum_head_seal, inner_relation_flags) in inner_relation_flags_m.iter() {
                // Just make sure that inner_relation_flags obeys the mask constraint.
                assert_eq!(*inner_relation_flags & !mask, PlumRelationFlags::NONE);

                // NOTE that we're passing mask here, instead of inner_relation_flags, meaning that the
                // full mask will "bypass" any RelationFlag "bottleneck" imposed by a particular data type.
                // For example, only CONTENT_DEPENDENCY is used by DirNode, but if mask includes
                // METADATA_DEPENDENCY, then on querying a child of the DirNode for its plum_relations,
                // METADATA_DEPENDENCY will be fair game again.  This may or may not be what is actually
                // desired.  Will determine through testing.
                log::trace!(
                    "accumulate_relations_recursive_impl; recursing on {}",
                    inner_plum_head_seal
                );
                self.accumulate_relations_recursive_impl(
                    &mut *transaction,
                    inner_plum_head_seal,
                    mask.clone(),
                    plum_relation_flags_m,
                )
                .await?;

                // Add inner_plum_head_seal with its computed inner_relation_flags to mark as traversed.
                log::trace!(
                "accumulate_relations_recursive_impl; adding to plum_relation_flags_m: {} -> {:?}",
                inner_plum_head_seal,
                inner_relation_flags
            );
                plum_relation_flags_m.insert(inner_plum_head_seal.clone(), *inner_relation_flags);
            }

            Ok(())
        })
    }

    //
    // Methods for fragment query
    //

    // TODO: Eventually make this return Box<Any> or something
    pub async fn fragment_query(
        &self,
        starting_plum_head_seal: &PlumHeadSeal,
        query_str: &str,
    ) -> Result<PlumHeadSeal> {
        let mut transaction_b = self.datahost_storage_b.begin_transaction().await?;

        let mut current_plum_head_seal = starting_plum_head_seal.clone();
        let mut current_query_str = query_str;
        loop {
            let plum_head = self
                .datahost_storage_b
                .load_plum_head(transaction_b.as_mut(), &current_plum_head_seal)
                .await?;
            let plum_body = self
                .datahost_storage_b
                .load_plum_body(transaction_b.as_mut(), &plum_head.plum_body_seal)
                .await?;
            let fragment_query_result =
                match std::str::from_utf8(plum_body.plum_body_content_type.as_ref()) {
                    // TODO: Replace this with a callback registry pattern
                    Ok("idp::BranchNode") => {
                        log::trace!("fragment_query; deserializing idp::BranchNode");
                        // if plum_body.plum_body_content_o.is_none() {
                        //     return Err(anyhow::format_err!(
                        //         "Plum {} had missing plum_body_content",
                        //         current_plum_head_seal
                        //     ));
                        // }
                        // Deserialize plum_body_content and call fragment_query_single_segment.
                        // let plum_body_content = plum_body.plum_body_content_o.unwrap();
                        let branch_node: BranchNode =
                            rmp_serde::from_read_ref(&plum_body.plum_body_content)?;
                        branch_node.fragment_query_single_segment(
                            &current_plum_head_seal,
                            current_query_str,
                        )?
                    }
                    Ok("idp::DirNode") => {
                        log::trace!("fragment_query; deserializing idp::BranchNode");
                        // if plum_body.plum_body_content_o.is_none() {
                        //     return Err(anyhow::format_err!(
                        //         "Plum {} had missing plum_body_content",
                        //         current_plum_head_seal
                        //     ));
                        // }
                        // Deserialize plum_body_content and call fragment_query_single_segment.
                        // let plum_body_content = plum_body.plum_body_content_o.unwrap();
                        let dir_node: DirNode =
                            rmp_serde::from_read_ref(&plum_body.plum_body_content)?;
                        dir_node.fragment_query_single_segment(
                            &current_plum_head_seal,
                            current_query_str,
                        )?
                    }
                    _ => {
                        // This data type is considered FragmentQueryable-opaque, so produce an error.
                        // Later, this should just return the plum_body_content.  But for now, for simplicity,
                        // the fragment query returns PlumHeadSeal.
                        return Err(anyhow::format_err!(
                        "not yet supported; This data type is considered FragmentQueryable-opaque"
                    ));
                    }
                };
            match fragment_query_result {
                FragmentQueryResult::Value(plum_head_seal) => {
                    // We reached the end of the query, so return.
                    return Ok(plum_head_seal);
                }
                FragmentQueryResult::ForwardQueryTo {
                    target,
                    rest_of_query_str,
                } => {
                    // The query must continue.
                    // This assert is to ensure the finite-time termination of this loop.
                    assert!(rest_of_query_str.len() < current_query_str.len());
                    // Update the "current" vars for the next iteration.
                    current_plum_head_seal = target;
                    current_query_str = rest_of_query_str;
                }
            }
        }
    }

    //
    // Methods for path-based state
    //

    // TODO: Move to somewhere appropriate
    pub async fn begin_transaction(
        &self,
    ) -> std::result::Result<Box<dyn DatahostStorageTransaction>, DatahostStorageError> {
        self.datahost_storage_b.begin_transaction().await
    }
    // TODO: This should probably return a DatahostError type instead of DatahostStorageError.
    pub async fn has_path_state(
        &self,
        path: &Path,
        transaction_o: Option<&mut dyn DatahostStorageTransaction>,
    ) -> std::result::Result<bool, DatahostStorageError> {
        // A trick for declaring a locally-scoped transaction if needed.
        // TODO: Make a private method in Datahost (or DatahostStorage?) for this.
        let mut locally_scoped_transaction_bo = if transaction_o.is_some() {
            None
        } else {
            Some(self.begin_transaction().await?)
        };
        let transaction = match (
            transaction_o,
            locally_scoped_transaction_bo
                .as_mut()
                .map(|transaction_b| transaction_b.as_mut()),
        ) {
            (Some(transaction), None) => transaction,
            (None, Some(transaction)) => transaction,
            _ => {
                panic!("programmer error: this case should be impossible");
            }
        };

        let retval = self
            .datahost_storage_b
            .has_path_state(transaction, path)
            .await?;

        if let Some(locally_scoped_transaction_b) = locally_scoped_transaction_bo {
            locally_scoped_transaction_b.commit().await?;
        }

        Ok(retval)
    }
    pub async fn load_path_state(
        &self,
        path: &Path,
        transaction_o: Option<&mut dyn DatahostStorageTransaction>,
    ) -> Result<PathState, DatahostStorageError> {
        // A trick for declaring a locally-scoped transaction if needed.
        // TODO: Make a private method in Datahost (or DatahostStorage?) for this.
        let mut locally_scoped_transaction_bo = if transaction_o.is_some() {
            None
        } else {
            Some(self.begin_transaction().await?)
        };
        let transaction = match (
            transaction_o,
            locally_scoped_transaction_bo
                .as_mut()
                .map(|transaction_b| transaction_b.as_mut()),
        ) {
            (Some(transaction), None) => transaction,
            (None, Some(transaction)) => transaction,
            _ => {
                panic!("programmer error: this case should be impossible");
            }
        };

        let retval = self
            .datahost_storage_b
            .load_path_state(transaction, path)
            .await?;

        if let Some(locally_scoped_transaction_b) = locally_scoped_transaction_bo {
            locally_scoped_transaction_b.commit().await?;
        }

        Ok(retval)
    }
    pub async fn insert_path_state(
        &self,
        path_state: &PathState,
        transaction_o: Option<&mut dyn DatahostStorageTransaction>,
    ) -> Result<(), DatahostStorageError> {
        // A trick for declaring a locally-scoped transaction if needed.
        // TODO: Make a private method in Datahost (or DatahostStorage?) for this.
        let mut locally_scoped_transaction_bo = if transaction_o.is_some() {
            None
        } else {
            Some(self.begin_transaction().await?)
        };
        let transaction = match (
            transaction_o,
            locally_scoped_transaction_bo
                .as_mut()
                .map(|transaction_b| transaction_b.as_mut()),
        ) {
            (Some(transaction), None) => transaction,
            (None, Some(transaction)) => transaction,
            _ => {
                panic!("programmer error: this case should be impossible");
            }
        };

        let retval = self
            .datahost_storage_b
            .insert_path_state(transaction, path_state)
            .await?;

        if let Some(locally_scoped_transaction_b) = locally_scoped_transaction_bo {
            locally_scoped_transaction_b.commit().await?;
        }

        Ok(retval)
    }
    pub async fn update_path_state(
        &self,
        path_state: &PathState,
        transaction_o: Option<&mut dyn DatahostStorageTransaction>,
    ) -> Result<(), DatahostStorageError> {
        // A trick for declaring a locally-scoped transaction if needed.
        // TODO: Make a private method in Datahost (or DatahostStorage?) for this.
        let mut locally_scoped_transaction_bo = if transaction_o.is_some() {
            None
        } else {
            Some(self.begin_transaction().await?)
        };
        let transaction = match (
            transaction_o,
            locally_scoped_transaction_bo
                .as_mut()
                .map(|transaction_b| transaction_b.as_mut()),
        ) {
            (Some(transaction), None) => transaction,
            (None, Some(transaction)) => transaction,
            _ => {
                panic!("programmer error: this case should be impossible");
            }
        };

        let retval = self
            .datahost_storage_b
            .update_path_state(transaction, path_state)
            .await?;

        if let Some(locally_scoped_transaction_b) = locally_scoped_transaction_bo {
            locally_scoped_transaction_b.commit().await?;
        }

        Ok(retval)
    }
    pub async fn delete_path_state(
        &self,
        path: &Path,
        transaction_o: Option<&mut dyn DatahostStorageTransaction>,
    ) -> Result<(), DatahostStorageError> {
        // A trick for declaring a locally-scoped transaction if needed.
        // TODO: Make a private method in Datahost (or DatahostStorage?) for this.
        let mut locally_scoped_transaction_bo = if transaction_o.is_some() {
            None
        } else {
            Some(self.begin_transaction().await?)
        };
        let transaction = match (
            transaction_o,
            locally_scoped_transaction_bo
                .as_mut()
                .map(|transaction_b| transaction_b.as_mut()),
        ) {
            (Some(transaction), None) => transaction,
            (None, Some(transaction)) => transaction,
            _ => {
                panic!("programmer error: this case should be impossible");
            }
        };

        let retval = self
            .datahost_storage_b
            .delete_path_state(transaction, path)
            .await?;

        if let Some(locally_scoped_transaction_b) = locally_scoped_transaction_bo {
            locally_scoped_transaction_b.commit().await?;
        }

        Ok(retval)
    }
}

impl Drop for Datahost {
    fn drop(&mut self) {
        log::info!("Datahost closed");
    }
}
