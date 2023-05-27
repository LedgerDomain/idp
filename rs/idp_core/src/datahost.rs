use crate::{
    BranchError, BranchNode, DirNode, FragmentQueryResult, FragmentQueryable,
    LoadPlumAndDeserializeError, PathStateError,
};
use anyhow::Result;
use idp_datahost_storage::{DatahostStorage, DatahostStorageError, DatahostStorageTransaction};
use idp_proto::{
    BranchSetHeadRequest, Path, PathState, Plum, PlumBody, PlumBodySeal, PlumHead, PlumHeadSeal,
    PlumRelationFlags, PlumRelations, PlumRelationsSeal,
};
use std::{collections::HashMap, convert::TryFrom};

pub struct Datahost {
    datahost_storage_b: Box<dyn DatahostStorage>,
}

impl Datahost {
    pub fn open(datahost_storage: impl DatahostStorage + 'static) -> Self {
        Self {
            datahost_storage_b: Box::new(datahost_storage),
        }
    }

    pub async fn begin_transaction(
        &self,
    ) -> Result<Box<dyn DatahostStorageTransaction>, DatahostStorageError> {
        self.datahost_storage_b.begin_transaction().await
    }

    //
    // Data methods
    //

    pub async fn has_plum_head(
        &self,
        plum_head_seal: &PlumHeadSeal,
        transaction_o: Option<&mut dyn DatahostStorageTransaction>,
    ) -> Result<bool, DatahostStorageError> {
        // Note that the self.datahost_storage_b.begin_transaction() simply returns a Future, it doesn't actually begin the transaction.
        let mut tx =
            EnsuredTransaction::new(transaction_o, self.datahost_storage_b.begin_transaction())
                .await?;
        let has_plum_head = self
            .datahost_storage_b
            .has_plum_head(tx.as_mut(), plum_head_seal)
            .await?;
        tx.finish().await?;
        Ok(has_plum_head)
    }
    pub async fn has_plum_relations(
        &self,
        plum_relations_seal: &PlumRelationsSeal,
        transaction_o: Option<&mut dyn DatahostStorageTransaction>,
    ) -> Result<bool, DatahostStorageError> {
        // Note that the self.datahost_storage_b.begin_transaction() simply returns a Future, it doesn't actually begin the transaction.
        let mut tx =
            EnsuredTransaction::new(transaction_o, self.datahost_storage_b.begin_transaction())
                .await?;
        let has_plum_relations = self
            .datahost_storage_b
            .has_plum_relations(tx.as_mut(), plum_relations_seal)
            .await?;
        tx.finish().await?;
        Ok(has_plum_relations)
    }
    pub async fn has_plum_body(
        &self,
        plum_body_seal: &PlumBodySeal,
        transaction_o: Option<&mut dyn DatahostStorageTransaction>,
    ) -> Result<bool, DatahostStorageError> {
        // Note that the self.datahost_storage_b.begin_transaction() simply returns a Future, it doesn't actually begin the transaction.
        let mut tx =
            EnsuredTransaction::new(transaction_o, self.datahost_storage_b.begin_transaction())
                .await?;
        let has_plum_body = self
            .datahost_storage_b
            .has_plum_body(tx.as_mut(), plum_body_seal)
            .await?;
        tx.finish().await?;
        Ok(has_plum_body)
    }
    pub async fn has_plum(
        &self,
        plum_head_seal: &PlumHeadSeal,
        transaction_o: Option<&mut dyn DatahostStorageTransaction>,
    ) -> Result<bool, DatahostStorageError> {
        // Note that the self.datahost_storage_b.begin_transaction() simply returns a Future, it doesn't actually begin the transaction.
        let mut tx =
            EnsuredTransaction::new(transaction_o, self.datahost_storage_b.begin_transaction())
                .await?;
        let has_plum = self
            .datahost_storage_b
            .has_plum(tx.as_mut(), plum_head_seal)
            .await?;
        tx.finish().await?;
        Ok(has_plum)
    }
    pub async fn store_plum_head(
        &self,
        plum_head: &PlumHead,
        transaction_o: Option<&mut dyn DatahostStorageTransaction>,
    ) -> Result<PlumHeadSeal> {
        log::trace!(
            "Datahost::store_plum_head; PlumHeadSeal is {}",
            PlumHeadSeal::from(plum_head)
        );

        // Note that the self.datahost_storage_b.begin_transaction() simply returns a Future, it doesn't actually begin the transaction.
        let mut tx =
            EnsuredTransaction::new(transaction_o, self.datahost_storage_b.begin_transaction())
                .await?;
        let plum_head_seal = self
            .datahost_storage_b
            .store_plum_head(tx.as_mut(), plum_head)
            .await?;
        tx.finish().await?;
        Ok(plum_head_seal)
    }

    pub async fn store_plum_relations(
        &self,
        plum_relations: &PlumRelations,
        transaction_o: Option<&mut dyn DatahostStorageTransaction>,
    ) -> Result<PlumRelationsSeal> {
        log::trace!(
            "Datahost::store_plum_relations; PlumRelationsSeal is {}",
            PlumRelationsSeal::from(plum_relations)
        );

        // Note that the self.datahost_storage_b.begin_transaction() simply returns a Future, it doesn't actually begin the transaction.
        let mut tx =
            EnsuredTransaction::new(transaction_o, self.datahost_storage_b.begin_transaction())
                .await?;
        let plum_relations_seal = self
            .datahost_storage_b
            .store_plum_relations(tx.as_mut(), plum_relations)
            .await?;
        tx.finish().await?;
        Ok(plum_relations_seal)
    }

    pub async fn store_plum_body(
        &self,
        plum_body: &PlumBody,
        transaction_o: Option<&mut dyn DatahostStorageTransaction>,
    ) -> Result<PlumBodySeal> {
        log::trace!(
            "Datahost::store_plum_body; PlumBodySeal is {}",
            PlumBodySeal::from(plum_body)
        );

        // Note that the self.datahost_storage_b.begin_transaction() simply returns a Future, it doesn't actually begin the transaction.
        let mut tx =
            EnsuredTransaction::new(transaction_o, self.datahost_storage_b.begin_transaction())
                .await?;
        let plum_body_seal = self
            .datahost_storage_b
            .store_plum_body(tx.as_mut(), plum_body)
            .await?;
        tx.finish().await?;
        Ok(plum_body_seal)
    }

    pub async fn store_plum(
        &self,
        plum: &Plum,
        transaction_o: Option<&mut dyn DatahostStorageTransaction>,
    ) -> Result<PlumHeadSeal> {
        log::debug!(
            "Datahost::store_plum; plum's PlumHeadSeal is {}",
            PlumHeadSeal::from(&plum.plum_head),
        );
        // Note that the self.datahost_storage_b.begin_transaction() simply returns a Future, it doesn't actually begin the transaction.
        let mut tx =
            EnsuredTransaction::new(transaction_o, self.datahost_storage_b.begin_transaction())
                .await?;
        let plum_head_seal = self
            .datahost_storage_b
            .store_plum(tx.as_mut(), plum)
            .await?;
        tx.finish().await?;
        Ok(plum_head_seal)
    }

    /// If the specified PlumHead doesn't exist in this Datahost, returns error.
    pub async fn load_plum_head(
        &self,
        plum_head_seal: &PlumHeadSeal,
        transaction_o: Option<&mut dyn DatahostStorageTransaction>,
    ) -> Result<PlumHead> {
        // Note that the self.datahost_storage_b.begin_transaction() simply returns a Future, it doesn't actually begin the transaction.
        let mut tx =
            EnsuredTransaction::new(transaction_o, self.datahost_storage_b.begin_transaction())
                .await?;
        let plum_head = self
            .datahost_storage_b
            .load_plum_head(tx.as_mut(), plum_head_seal)
            .await?;
        tx.finish().await?;
        Ok(plum_head)
    }

    /// If either of the PlumHead or PlumBody for the specified Plum doesn't exist in this Datahost,
    /// returns None.
    pub async fn load_option_plum(
        &self,
        plum_head_seal: &PlumHeadSeal,
        transaction_o: Option<&mut dyn DatahostStorageTransaction>,
    ) -> Result<Option<Plum>> {
        // Note that the self.datahost_storage_b.begin_transaction() simply returns a Future, it doesn't actually begin the transaction.
        let mut tx =
            EnsuredTransaction::new(transaction_o, self.datahost_storage_b.begin_transaction())
                .await?;
        let plum_o = self
            .datahost_storage_b
            .load_option_plum(tx.as_mut(), plum_head_seal)
            .await?;
        tx.finish().await?;
        Ok(plum_o)
    }
    pub async fn load_plum(
        &self,
        plum_head_seal: &PlumHeadSeal,
        transaction_o: Option<&mut dyn DatahostStorageTransaction>,
    ) -> Result<Plum> {
        // Note that the self.datahost_storage_b.begin_transaction() simply returns a Future, it doesn't actually begin the transaction.
        let mut tx =
            EnsuredTransaction::new(transaction_o, self.datahost_storage_b.begin_transaction())
                .await?;
        let plum = self
            .datahost_storage_b
            .load_plum(tx.as_mut(), plum_head_seal)
            .await?;
        tx.finish().await?;
        Ok(plum)
    }
    /// Load the specified Plum, check the PlumBody's ContentType against that expected by T,
    /// and then deserialize the PlumBody content into T.
    // TODO: Consider having it return the Plum or PlumHead as well, potentially deserializing
    // any PlumHead metadata into another type.
    pub async fn load_plum_and_decode_and_deserialize<T: idp_proto::Deserializable>(
        &self,
        plum_head_seal: &PlumHeadSeal,
        transaction_o: Option<&mut dyn DatahostStorageTransaction>,
    ) -> std::result::Result<T, LoadPlumAndDeserializeError> {
        let plum = self
            .load_plum(&plum_head_seal, transaction_o)
            .await
            .map_err(|_| LoadPlumAndDeserializeError::FailedToLoadPlum)?;
        idp_proto::decode_and_deserialize_from_content(&plum.plum_body.plum_body_content)
            .map_err(|_| LoadPlumAndDeserializeError::DeserializationError)
    }

    //
    // Methods for determining plum_relations between Plums
    //

    pub async fn accumulated_relations_recursive(
        &self,
        plum_head_seal: &PlumHeadSeal,
        mask: PlumRelationFlags,
        transaction_o: Option<&mut dyn DatahostStorageTransaction>,
    ) -> Result<HashMap<PlumHeadSeal, PlumRelationFlags>> {
        // Note that the self.datahost_storage_b.begin_transaction() simply returns a Future, it doesn't actually begin the transaction.
        let mut tx =
            EnsuredTransaction::new(transaction_o, self.datahost_storage_b.begin_transaction())
                .await?;
        let mut plum_relation_flags_m = HashMap::new();
        self.accumulate_relations_recursive_impl(
            tx.as_mut(),
            plum_head_seal,
            mask,
            &mut plum_relation_flags_m,
        )
        .await?;
        tx.finish().await?;
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
                let plum_relations = self
                    .datahost_storage_b
                    .load_plum_relations(&mut *transaction, &plum_head.plum_relations_seal)
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
        transaction_o: Option<&mut dyn DatahostStorageTransaction>,
    ) -> Result<PlumHeadSeal> {
        // Note that the self.datahost_storage_b.begin_transaction() simply returns a Future, it doesn't actually begin the transaction.
        let mut tx =
            EnsuredTransaction::new(transaction_o, self.datahost_storage_b.begin_transaction())
                .await?;
        let retval = self
            .fragment_query_internal(starting_plum_head_seal, query_str, tx.as_mut())
            .await?;
        tx.finish().await?;
        Ok(retval)
    }

    // TODO: Eventually make this return Box<Any> or something
    pub async fn fragment_query_internal(
        &self,
        starting_plum_head_seal: &PlumHeadSeal,
        query_str: &str,
        transaction: &mut dyn DatahostStorageTransaction,
    ) -> Result<PlumHeadSeal> {
        let mut current_plum_head_seal = starting_plum_head_seal.clone();
        let mut current_query_str = query_str;
        loop {
            let plum_head = self
                .datahost_storage_b
                .load_plum_head(transaction, &current_plum_head_seal)
                .await?;
            let plum_body = self
                .datahost_storage_b
                .load_plum_body(transaction, &plum_head.plum_body_seal)
                .await?;
            use idp_proto::ContentClassifiable;
            let fragment_query_result = match plum_body
                .plum_body_content
                .content_metadata
                .content_class
                .as_str()
            {
                // TODO: Replace this with a callback registry pattern
                s if s == BranchNode::content_class_str() => {
                    log::trace!("fragment_query; deserializing idp::BranchNode");
                    // Deserialize plum_body_content and call fragment_query_single_segment.
                    let branch_node: BranchNode = idp_proto::decode_and_deserialize_from_content(
                        &plum_body.plum_body_content,
                    )?;
                    branch_node
                        .fragment_query_single_segment(&current_plum_head_seal, current_query_str)?
                }
                s if s == DirNode::content_class_str() => {
                    log::trace!("fragment_query; deserializing idp::DirNode");
                    // Deserialize plum_body_content and call fragment_query_single_segment.
                    let dir_node: DirNode = idp_proto::decode_and_deserialize_from_content(
                        &plum_body.plum_body_content,
                    )?;
                    dir_node
                        .fragment_query_single_segment(&current_plum_head_seal, current_query_str)?
                }
                _ => {
                    // This data type is considered FragmentQueryable-opaque, so produce an error.
                    // Later, this should just return the plum_body_content.  But for now, for simplicity,
                    // the fragment query returns PlumHeadSeal.
                    return Err(anyhow::anyhow!(
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

    // TODO: This should probably return a DatahostError type instead of DatahostStorageError.
    pub async fn has_path_state(
        &self,
        path: &Path,
        transaction_o: Option<&mut dyn DatahostStorageTransaction>,
    ) -> std::result::Result<bool, DatahostStorageError> {
        log::trace!("Datahost::has_path_state({:?})", path);
        // Note that the self.datahost_storage_b.begin_transaction() simply returns a Future, it doesn't actually begin the transaction.
        let mut tx =
            EnsuredTransaction::new(transaction_o, self.datahost_storage_b.begin_transaction())
                .await?;
        let retval = self
            .datahost_storage_b
            .has_path_state(tx.as_mut(), path)
            .await?;
        tx.finish().await?;
        Ok(retval)
    }
    pub async fn load_path_state(
        &self,
        path: &Path,
        transaction_o: Option<&mut dyn DatahostStorageTransaction>,
    ) -> Result<PathState, DatahostStorageError> {
        log::trace!("Datahost::load_path_state({:?})", path);
        // Note that the self.datahost_storage_b.begin_transaction() simply returns a Future, it doesn't actually begin the transaction.
        let mut tx =
            EnsuredTransaction::new(transaction_o, self.datahost_storage_b.begin_transaction())
                .await?;
        let retval = self
            .datahost_storage_b
            .load_path_state(tx.as_mut(), path)
            .await?;
        tx.finish().await?;
        Ok(retval)
    }
    pub async fn insert_path_state(
        &self,
        path_state: &PathState,
        transaction_o: Option<&mut dyn DatahostStorageTransaction>,
    ) -> Result<(), DatahostStorageError> {
        log::trace!(
            "Datahost::insert_path_state({:?} -> {})",
            path_state.path,
            path_state.current_state_plum_head_seal
        );
        // Note that the self.datahost_storage_b.begin_transaction() simply returns a Future, it doesn't actually begin the transaction.
        let mut tx =
            EnsuredTransaction::new(transaction_o, self.datahost_storage_b.begin_transaction())
                .await?;
        self.datahost_storage_b
            .insert_path_state(tx.as_mut(), path_state)
            .await?;
        tx.finish().await?;
        Ok(())
    }
    pub async fn update_path_state(
        &self,
        path_state: &PathState,
        transaction_o: Option<&mut dyn DatahostStorageTransaction>,
    ) -> Result<(), DatahostStorageError> {
        log::trace!(
            "Datahost::update_path_state({:?} -> {})",
            path_state.path,
            path_state.current_state_plum_head_seal
        );
        // Note that the self.datahost_storage_b.begin_transaction() simply returns a Future, it doesn't actually begin the transaction.
        let mut tx =
            EnsuredTransaction::new(transaction_o, self.datahost_storage_b.begin_transaction())
                .await?;
        self.datahost_storage_b
            .update_path_state(tx.as_mut(), path_state)
            .await?;
        tx.finish().await?;
        Ok(())
    }
    pub async fn delete_path_state(
        &self,
        path: &Path,
        transaction_o: Option<&mut dyn DatahostStorageTransaction>,
    ) -> Result<(), DatahostStorageError> {
        log::trace!("Datahost::delete_path_state({:?})", path);
        // Note that the self.datahost_storage_b.begin_transaction() simply returns a Future, it doesn't actually begin the transaction.
        let mut tx =
            EnsuredTransaction::new(transaction_o, self.datahost_storage_b.begin_transaction())
                .await?;
        let retval = self
            .datahost_storage_b
            .delete_path_state(tx.as_mut(), path)
            .await?;
        tx.finish().await?;
        Ok(retval)
    }

    //
    // Methods for Branch operations
    //

    pub async fn branch_create(
        &self,
        branch_path_state: &PathState,
        transaction_o: Option<&mut dyn DatahostStorageTransaction>,
    ) -> std::result::Result<(), BranchError> {
        log::trace!(
            "Datahost::branch_create({:?} -> {})",
            branch_path_state.path,
            branch_path_state.current_state_plum_head_seal
        );

        branch_path_state
            .path
            .validate()
            .map_err(|e| PathStateError::InvalidPath {
                path: branch_path_state.path.clone(),
                reason: e,
            })?;

        // TODO: Any authorization checks for creating a branch with the given path

        // let mut transaction_b = self.datahost_storage_b.begin_transaction().await?;

        // Note that the self.datahost_storage_b.begin_transaction() simply returns a Future, it doesn't actually begin the transaction.
        let mut tx =
            EnsuredTransaction::new(transaction_o, self.datahost_storage_b.begin_transaction())
                .await?;

        // Check if the PathState already exists.
        if self
            .has_path_state(&branch_path_state.path, Some(tx.as_mut()))
            .await?
        {
            return Err(PathStateError::PathAlreadyExists(branch_path_state.path.clone()).into());
        }

        // Check that the BranchNode Plum already has already been pushed.
        if !self
            .has_plum(
                &branch_path_state.current_state_plum_head_seal,
                Some(tx.as_mut()),
            )
            .await?
        {
            return Err(PathStateError::PlumMustAlreadyExist(
                branch_path_state.current_state_plum_head_seal.clone(),
            )
            .into());
        }
        // TODO: Check that req.branch_path_state.current_state_plum_head_seal is dependency-complete.

        // TODO: Move this BranchNode validation stuff into helper function

        // Check that the BranchNode Plum is actually a BranchNode.
        let branch_node_plum = self
            .load_plum(
                &branch_path_state.current_state_plum_head_seal,
                Some(tx.as_mut()),
            )
            .await
            .map_err(|e| BranchError::InternalError {
                description: e.to_string(),
            })?;
        // NOTE: This particular check is actually done by idp_proto::decode_and_deserialize_from_content,
        // but we do it here in order to return a BranchError.  This could be improved if
        // idp_proto::decode_and_deserialize_from_content had its own formal error type with details.
        use idp_proto::ContentClassifiable;
        if branch_node_plum
            .plum_body
            .plum_body_content
            .content_metadata
            .content_class
            .as_str()
            != BranchNode::content_class_str()
        {
            return Err(BranchError::PlumIsNotABranchNode {
                plum_head_seal: branch_path_state.current_state_plum_head_seal.clone(),
                description: "PlumBody content type was not \"idp::BranchNode\"".to_string(),
            });
        }
        let _branch_node: BranchNode = idp_proto::decode_and_deserialize_from_content(
            &branch_node_plum.plum_body.plum_body_content,
        )
        .map_err(|e| BranchError::PlumIsNotABranchNode {
            plum_head_seal: branch_path_state.current_state_plum_head_seal.clone(),
            description: format!(
                "PlumBody content failed to decode and deserialize into BranchNode; {}",
                e
            ),
        })?;

        // The BranchNode Plum has been validated.  It can be stored now.
        self.insert_path_state(&branch_path_state, Some(tx.as_mut()))
            .await?;

        tx.finish().await?;

        Ok(())
    }
    pub async fn branch_delete(
        &self,
        branch_path: &Path,
        transaction_o: Option<&mut dyn DatahostStorageTransaction>,
    ) -> Result<(), BranchError> {
        branch_path
            .validate()
            .map_err(|e| PathStateError::InvalidPath {
                path: branch_path.clone(),
                reason: e,
            })?;

        // TODO: Any authorization checks for deleting the branch.

        // Note that the self.datahost_storage_b.begin_transaction() simply returns a Future, it doesn't actually begin the transaction.
        let mut tx =
            EnsuredTransaction::new(transaction_o, self.datahost_storage_b.begin_transaction())
                .await?;
        self.delete_path_state(branch_path, Some(tx.as_mut()))
            .await?;
        tx.finish().await?;
        Ok(())
    }
    pub async fn branch_get_head(
        &self,
        branch_path: &Path,
        transaction_o: Option<&mut dyn DatahostStorageTransaction>,
    ) -> Result<PlumHeadSeal, BranchError> {
        branch_path
            .validate()
            .map_err(|e| PathStateError::InvalidPath {
                path: branch_path.clone(),
                reason: e,
            })?;

        // TODO: Any authorization checks for getting the branch head

        // Note that the self.datahost_storage_b.begin_transaction() simply returns a Future, it doesn't actually begin the transaction.
        let mut tx =
            EnsuredTransaction::new(transaction_o, self.datahost_storage_b.begin_transaction())
                .await?;
        // Query storage for the current PlumHeadSeal of the given path
        let path_state = self.load_path_state(branch_path, Some(tx.as_mut())).await?;
        tx.finish().await?;
        Ok(path_state.current_state_plum_head_seal)
    }
    pub async fn branch_set_head(
        &self,
        // NOTE: This would be split up if/when idp API is separated from GRPC.
        req: BranchSetHeadRequest,
        transaction_o: Option<&mut dyn DatahostStorageTransaction>,
    ) -> Result<(), BranchError> {
        req.branch_path
            .validate()
            .map_err(|e| PathStateError::InvalidPath {
                path: req.branch_path.clone(),
                reason: e,
            })?;

        // Any authorization checks for the given path

        let req_value = req.value.ok_or_else(|| BranchError::MalformedRequest {
            description: "req.value is None".to_string(),
        })?;
        let new_branch_head_plum_head_seal = match &req_value {
            idp_proto::branch_set_head_request::Value::BranchFastForwardTo(plum_head_seal) => {
                plum_head_seal.clone()
            }
            idp_proto::branch_set_head_request::Value::BranchRewindTo(plum_head_seal) => {
                plum_head_seal.clone()
            }
            idp_proto::branch_set_head_request::Value::BranchForkHistoryTo(plum_head_seal) => {
                plum_head_seal.clone()
            }
            idp_proto::branch_set_head_request::Value::BranchTotallyRewriteTo(plum_head_seal) => {
                plum_head_seal.clone()
            }
        };

        // Note that the self.datahost_storage_b.begin_transaction() simply returns a Future, it doesn't actually begin the transaction.
        let mut tx =
            EnsuredTransaction::new(transaction_o, self.datahost_storage_b.begin_transaction())
                .await?;

        // Check that the BranchNode Plum already has already been pushed.
        if !self
            .has_plum(&new_branch_head_plum_head_seal, Some(tx.as_mut()))
            .await?
        {
            return Err(BranchError::BranchNodePlumMustAlreadyExist(
                new_branch_head_plum_head_seal,
            ));
        }
        // TODO: Check that branch_node_plum_head_seal is dependency-complete.

        // Get the current branch PlumHeadSeal.
        let current_branch_head_plum_head_seal = self
            .load_path_state(&req.branch_path, Some(tx.as_mut()))
            .await?
            .current_state_plum_head_seal;

        // TODO: Move this BranchNode validation stuff into helper function

        // Check that the BranchNode Plum is actually a BranchNode.
        // TODO: Can do this via load_plum_and_decode_and_deserialize
        let new_branch_head_plum = self
            .load_plum(&new_branch_head_plum_head_seal, Some(tx.as_mut()))
            .await
            .map_err(|e| BranchError::InternalError {
                description: e.to_string(),
            })?;
        // NOTE: This particular check is actually done by idp_proto::decode_and_deserialize_from_content,
        // but we do it here in order to return a BranchError.  This could be improved if
        // idp_proto::decode_and_deserialize_from_content had its own formal error type with details.
        use idp_proto::ContentClassifiable;
        if new_branch_head_plum
            .plum_body
            .plum_body_content
            .content_metadata
            .content_class
            .as_str()
            != BranchNode::content_class_str()
        {
            return Err(BranchError::PlumIsNotABranchNode {
                plum_head_seal: new_branch_head_plum_head_seal,
                description: "PlumBody content type was not \"idp::BranchNode\"".to_string(),
            });
        }
        let _new_branch_head: BranchNode = idp_proto::decode_and_deserialize_from_content(
            &new_branch_head_plum.plum_body.plum_body_content,
        )
        .map_err(|e| BranchError::PlumIsNotABranchNode {
            plum_head_seal: new_branch_head_plum_head_seal.clone(),
            description: format!(
                "PlumBody content failed to decode and deserialize into BranchNode; {}",
                e
            ),
        })?;

        // The BranchNode Plum has been validated.  Now check the validity of the branch operation.
        // If it's a fast-forward, check that the history of the specified Plum includes the current branch head.
        // If it's a rewind, check that the specified Plum is in the history of the current branch head.
        // If it's a fork history, check that there is a common ancestor between the specified Plum and the current branch head
        // If it's a total rewrite, check that there is no common ancestor, since this is a stronger operation.
        let common_ancestor_o = self
            .closest_common_branch_node_ancestor(
                &current_branch_head_plum_head_seal,
                &new_branch_head_plum_head_seal,
                Some(tx.as_mut()),
            )
            .await?;
        match req_value {
            idp_proto::branch_set_head_request::Value::BranchFastForwardTo(_) => {
                if common_ancestor_o.as_ref() != Some(&current_branch_head_plum_head_seal) {
                    return Err(BranchError::FastForwardExpectedDescendant {
                        current_branch_head: current_branch_head_plum_head_seal,
                        new_branch_head: new_branch_head_plum_head_seal,
                    });
                }
            }
            idp_proto::branch_set_head_request::Value::BranchRewindTo(_) => {
                if common_ancestor_o.as_ref() != Some(&new_branch_head_plum_head_seal) {
                    return Err(BranchError::RewindExpectedAncestor {
                        current_branch_head: current_branch_head_plum_head_seal,
                        new_branch_head: new_branch_head_plum_head_seal,
                    });
                }
            }
            idp_proto::branch_set_head_request::Value::BranchForkHistoryTo(_) => {
                if common_ancestor_o.is_none() {
                    return Err(BranchError::ForkHistoryExpectedCommonAncestor {
                        current_branch_head: current_branch_head_plum_head_seal,
                        new_branch_head: new_branch_head_plum_head_seal,
                    });
                }
            }
            idp_proto::branch_set_head_request::Value::BranchTotallyRewriteTo(_) => {
                if common_ancestor_o.is_some() {
                    return Err(BranchError::TotalRewriteExpectedNoCommonAncestor {
                        current_branch_head: current_branch_head_plum_head_seal,
                        new_branch_head: new_branch_head_plum_head_seal,
                    });
                }
            }
        }

        // The operation has been validated, so now go ahead and update the branch PathState.
        self.update_path_state(
            &PathState {
                path: req.branch_path,
                current_state_plum_head_seal: new_branch_head_plum_head_seal,
            },
            Some(tx.as_mut()),
        )
        .await?;

        tx.finish().await?;

        Ok(())
    }

    pub async fn closest_common_branch_node_ancestor(
        &self,
        lhs: &PlumHeadSeal,
        rhs: &PlumHeadSeal,
        transaction_o: Option<&mut dyn DatahostStorageTransaction>,
    ) -> Result<Option<PlumHeadSeal>, BranchError> {
        if lhs == rhs {
            return Ok(Some(lhs.clone()));
        }

        // Note that the self.datahost_storage_b.begin_transaction() simply returns a Future, it doesn't actually begin the transaction.
        let mut tx =
            EnsuredTransaction::new(transaction_o, self.datahost_storage_b.begin_transaction())
                .await?;

        // This is not optimal by any means, but who cares for now.
        let mut lhs_ancestor_s = maplit::hashset! { lhs.clone() };
        let mut rhs_ancestor_s = maplit::hashset! { rhs.clone() };
        let mut lhs_current_o = Some(lhs.clone());
        let mut rhs_current_o = Some(rhs.clone());

        // Somewhat arbitrary upper limit on branch length.  This is to guarantee that this operation
        // terminates.  Because (for now), PlumHeadSeal-s are hashes of content, it's computationally
        // infeasible to construct a valid cycle of dependencies.  However, the DB could be improperly
        // modified outside of this process to have such a cycle, so that condition must still be handled.
        // This is hardcoded to 2^20 = 1,048,576, which should be more than enough for now.  Potentially
        // this could be a param in this method.
        const MAX_ANCESTOR_DEPTH: usize = 1usize << 20;
        for _ in 0..MAX_ANCESTOR_DEPTH {
            // Find ancestor for each of current_lhs_o and current_rhs_o, if they're each not None.

            // TODO: Should be able to specify a Datacache for these load and deserialize operations,
            // and mark these cache entries as "expire soon". Could potentially do this using PlumRef.
            let lhs_ancestor_o = if let Some(lhs_current) = lhs_current_o {
                if rhs_ancestor_s.contains(&lhs_current) {
                    return Ok(Some(lhs_current));
                }
                // Load the ancestor.
                self.load_plum_and_decode_and_deserialize::<BranchNode>(
                    &lhs_current,
                    Some(tx.as_mut()),
                )
                .await
                .map_err(|e| BranchError::PlumIsNotABranchNode {
                    plum_head_seal: lhs_current,
                    description: e.to_string(),
                })?
                .ancestor_o
            } else {
                None
            };
            let rhs_ancestor_o = if let Some(rhs_current) = rhs_current_o {
                if lhs_ancestor_s.contains(&rhs_current) {
                    return Ok(Some(rhs_current));
                }
                // Load the ancestor.
                self.load_plum_and_decode_and_deserialize::<BranchNode>(
                    &rhs_current,
                    Some(tx.as_mut()),
                )
                .await
                .map_err(|e| BranchError::PlumIsNotABranchNode {
                    plum_head_seal: rhs_current,
                    description: e.to_string(),
                })?
                .ancestor_o
            } else {
                None
            };

            if lhs_ancestor_o.is_none() && rhs_ancestor_o.is_none() {
                // No more ancestors, and we haven't found one, so there is no common ancestor.
                return Ok(None);
            }

            // Update the ancestor sets
            if let Some(lhs_ancestor) = lhs_ancestor_o.as_ref() {
                lhs_ancestor_s.insert(lhs_ancestor.clone());
            }
            if let Some(rhs_ancestor) = rhs_ancestor_o.as_ref() {
                rhs_ancestor_s.insert(rhs_ancestor.clone());
            }

            // Update the cursors.
            lhs_current_o = lhs_ancestor_o;
            rhs_current_o = rhs_ancestor_o;
        }

        // If we exited the loop, that means the MAX_ANCESTOR_DEPTH was reached, which practically means either
        // - There was a programmer error, or
        // - The Datahost DB was improperly modified outside of this process to have a cycle.
        return Err(BranchError::MaxAncestorDepthReached {
            lhs: lhs.clone(),
            rhs: rhs.clone(),
        });
    }
}

impl Drop for Datahost {
    fn drop(&mut self) {
        log::info!("Datahost closed");
    }
}

/// Wrapper for ensuring properly transactional operations, allowing for a transaction to be passed
/// in from an outer scope, or generated and used for the inner scope of a function.
// TODO: This really should be associated with DatahostStorage, not Datahost.
pub struct EnsuredTransaction<'a> {
    outer_transaction_o: Option<&'a mut dyn DatahostStorageTransaction>,
    inner_transaction_bo: Option<Box<dyn DatahostStorageTransaction>>,
}

impl<'a> EnsuredTransaction<'a> {
    pub async fn new(
        outer_transaction_o: Option<&'a mut dyn DatahostStorageTransaction>,
        begin_transaction: impl std::future::Future<
            Output = Result<Box<dyn DatahostStorageTransaction>, DatahostStorageError>,
        >,
    ) -> Result<EnsuredTransaction<'a>, DatahostStorageError> {
        let inner_transaction_bo = if outer_transaction_o.is_some() {
            None
        } else {
            Some(begin_transaction.await?)
        };
        Ok(Self {
            outer_transaction_o,
            inner_transaction_bo,
        })
    }
    // TODO: Figure out if this should be DerefMut or BorrowMut or something
    pub fn as_mut(&mut self) -> &mut dyn DatahostStorageTransaction {
        match (
            &mut self.outer_transaction_o,
            self.inner_transaction_bo
                .as_mut()
                .map(|transaction_b| transaction_b.as_mut()),
        ) {
            (Some(transaction), None) => *transaction,
            (None, Some(transaction)) => transaction,
            _ => {
                panic!("programmer error: this case should be impossible");
            }
        }
    }
    /// If self.inner_transaction_bo.is_some(), then this commits.  Otherwise it does nothing.
    pub async fn finish(self) -> Result<(), DatahostStorageError> {
        if let Some(inner_transaction_b) = self.inner_transaction_bo {
            inner_transaction_b.commit().await?;
        }
        Ok(())
    }
}
