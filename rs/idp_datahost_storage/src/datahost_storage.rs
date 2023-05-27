use crate::{DatahostStorageError, DatahostStorageTransaction};
use idp_proto::{
    Path, PathState, Plum, PlumBody, PlumBodySeal, PlumHead, PlumHeadSeal, PlumMetadata,
    PlumMetadataSeal, PlumRelations, PlumRelationsSeal,
};

#[async_trait::async_trait]
pub trait DatahostStorage: Send + Sync {
    /// Begin a transaction, returning a transaction guard object that should be used to commit
    /// (or rollback, which is what happens when the object is dropped).
    async fn begin_transaction(
        &self,
    ) -> Result<Box<dyn DatahostStorageTransaction>, DatahostStorageError>;

    async fn has_plum_head(
        &self,
        transaction: &mut dyn DatahostStorageTransaction,
        plum_head_seal: &PlumHeadSeal,
    ) -> Result<bool, DatahostStorageError>;
    async fn has_plum_metadata(
        &self,
        transaction: &mut dyn DatahostStorageTransaction,
        plum_metadata_seal: &PlumMetadataSeal,
    ) -> Result<bool, DatahostStorageError>;
    async fn has_plum_relations(
        &self,
        transaction: &mut dyn DatahostStorageTransaction,
        plum_relations_seal: &PlumRelationsSeal,
    ) -> Result<bool, DatahostStorageError>;
    async fn has_plum_body(
        &self,
        transaction: &mut dyn DatahostStorageTransaction,
        plum_body_seal: &PlumBodySeal,
    ) -> Result<bool, DatahostStorageError>;
    async fn has_plum(
        &self,
        transaction: &mut dyn DatahostStorageTransaction,
        plum_head_seal: &PlumHeadSeal,
    ) -> Result<bool, DatahostStorageError> {
        let plum_head_o = self
            .load_option_plum_head(transaction, plum_head_seal)
            .await?;
        if plum_head_o.is_none() {
            return Ok(false);
        }
        let plum_head = plum_head_o.unwrap();

        if !self
            .has_plum_metadata(transaction, &plum_head.plum_metadata_seal)
            .await?
        {
            return Ok(false);
        }

        if !self
            .has_plum_relations(transaction, &plum_head.plum_relations_seal)
            .await?
        {
            return Ok(false);
        }

        if !self
            .has_plum_body(transaction, &plum_head.plum_body_seal)
            .await?
        {
            return Ok(false);
        }

        Ok(true)
    }

    async fn store_plum_head(
        &self,
        transaction: &mut dyn DatahostStorageTransaction,
        plum_head: &PlumHead,
    ) -> Result<PlumHeadSeal, DatahostStorageError>;
    async fn store_plum_metadata(
        &self,
        transaction: &mut dyn DatahostStorageTransaction,
        plum_metadata: &PlumMetadata,
    ) -> Result<PlumMetadataSeal, DatahostStorageError>;
    async fn store_plum_relations(
        &self,
        transaction: &mut dyn DatahostStorageTransaction,
        plum_relations: &PlumRelations,
    ) -> Result<PlumRelationsSeal, DatahostStorageError>;
    async fn store_plum_body(
        &self,
        transaction: &mut dyn DatahostStorageTransaction,
        plum_body: &PlumBody,
    ) -> Result<PlumBodySeal, DatahostStorageError>;

    async fn store_plum(
        &self,
        transaction: &mut dyn DatahostStorageTransaction,
        plum: &Plum,
    ) -> Result<PlumHeadSeal, DatahostStorageError> {
        // Verify the Plum before storing anything.
        plum.verify()?;

        // Now store its components.  Note that this computes the seals redundantly, which is not ideal, but is fine for now.
        self.store_plum_metadata(transaction, &plum.plum_metadata)
            .await?;
        self.store_plum_relations(transaction, &plum.plum_relations)
            .await?;
        self.store_plum_body(transaction, &plum.plum_body).await?;

        // Storing the plum head last ensures that the plum is fully stored before we commit to it.
        let plum_head_seal = self.store_plum_head(transaction, &plum.plum_head).await?;

        Ok(plum_head_seal)
    }

    async fn load_option_plum_head(
        &self,
        transaction: &mut dyn DatahostStorageTransaction,
        plum_head_seal: &PlumHeadSeal,
    ) -> Result<Option<PlumHead>, DatahostStorageError>;
    async fn load_option_plum_metadata(
        &self,
        transaction: &mut dyn DatahostStorageTransaction,
        plum_metadata_seal: &PlumMetadataSeal,
    ) -> Result<Option<PlumMetadata>, DatahostStorageError>;
    async fn load_option_plum_relations(
        &self,
        transaction: &mut dyn DatahostStorageTransaction,
        plum_relations_seal: &PlumRelationsSeal,
    ) -> Result<Option<PlumRelations>, DatahostStorageError>;
    async fn load_option_plum_body(
        &self,
        transaction: &mut dyn DatahostStorageTransaction,
        plum_body_seal: &PlumBodySeal,
    ) -> Result<Option<PlumBody>, DatahostStorageError>;
    async fn load_plum_head(
        &self,
        transaction: &mut dyn DatahostStorageTransaction,
        plum_head_seal: &PlumHeadSeal,
    ) -> Result<PlumHead, DatahostStorageError> {
        self.load_option_plum_head(transaction, plum_head_seal)
            .await?
            .ok_or_else(|| DatahostStorageError::PlumHeadNotFound(plum_head_seal.clone()))
    }
    async fn load_plum_metadata(
        &self,
        transaction: &mut dyn DatahostStorageTransaction,
        plum_metadata_seal: &PlumMetadataSeal,
    ) -> Result<PlumMetadata, DatahostStorageError> {
        self.load_option_plum_metadata(transaction, plum_metadata_seal)
            .await?
            .ok_or_else(|| DatahostStorageError::PlumMetadataNotFound(plum_metadata_seal.clone()))
    }
    async fn load_plum_relations(
        &self,
        transaction: &mut dyn DatahostStorageTransaction,
        plum_relations_seal: &PlumRelationsSeal,
    ) -> Result<PlumRelations, DatahostStorageError> {
        self.load_option_plum_relations(transaction, plum_relations_seal)
            .await?
            .ok_or_else(|| DatahostStorageError::PlumRelationsNotFound(plum_relations_seal.clone()))
    }
    async fn load_plum_body(
        &self,
        transaction: &mut dyn DatahostStorageTransaction,
        plum_body_seal: &PlumBodySeal,
    ) -> Result<PlumBody, DatahostStorageError> {
        self.load_option_plum_body(transaction, plum_body_seal)
            .await?
            .ok_or_else(|| DatahostStorageError::PlumBodyNotFound(plum_body_seal.clone()))
    }
    /// If any of the expected components (PlumHead, PlumMetadata, PlumRelations, or PlumBody) are missing,
    /// then this returns None.  Otherwise returns Some(plum), where plum is the Plum with those components.
    async fn load_option_plum(
        &self,
        transaction: &mut dyn DatahostStorageTransaction,
        plum_head_seal: &PlumHeadSeal,
    ) -> Result<Option<Plum>, DatahostStorageError> {
        let plum_head_o = self
            .load_option_plum_head(transaction, plum_head_seal)
            .await?;
        if plum_head_o.is_none() {
            return Ok(None);
        }
        let plum_head = plum_head_o.unwrap();

        let plum_metadata_o = self
            .load_option_plum_metadata(transaction, &plum_head.plum_metadata_seal)
            .await?;
        if plum_metadata_o.is_none() {
            return Ok(None);
        }
        let plum_metadata = plum_metadata_o.unwrap();

        let plum_relations_o = self
            .load_option_plum_relations(transaction, &plum_head.plum_relations_seal)
            .await?;
        if plum_relations_o.is_none() {
            return Ok(None);
        }
        let plum_relations = plum_relations_o.unwrap();

        let plum_body_o = self
            .load_option_plum_body(transaction, &plum_head.plum_body_seal)
            .await?;
        if plum_body_o.is_none() {
            return Ok(None);
        }
        let plum_body = plum_body_o.unwrap();

        // Construct the Plum and verify it before returning.  Even though only verified Plums have been stored
        // in the DB, this verification process could fail if the DB has been altered by an external process.
        let plum = Plum {
            plum_head,
            plum_metadata,
            plum_relations,
            plum_body,
        };
        plum.verify()?;

        Ok(Some(plum))
    }
    async fn load_plum(
        &self,
        transaction: &mut dyn DatahostStorageTransaction,
        plum_head_seal: &PlumHeadSeal,
    ) -> Result<Plum, DatahostStorageError> {
        let plum_head = self.load_plum_head(transaction, plum_head_seal).await?;
        let plum_metadata = self
            .load_plum_metadata(transaction, &plum_head.plum_metadata_seal)
            .await?;
        let plum_relations = self
            .load_plum_relations(transaction, &plum_head.plum_relations_seal)
            .await?;
        let plum_body = self
            .load_plum_body(transaction, &plum_head.plum_body_seal)
            .await?;

        // Construct the Plum and verify it before returning.  Even though only verified Plums have been stored
        // in the DB, this verification process could fail if the DB has been altered by an external process.
        let plum = Plum {
            plum_head,
            plum_metadata,
            plum_relations,
            plum_body,
        };
        plum.verify()?;

        Ok(plum)
    }

    async fn has_path_state(
        &self,
        transaction: &mut dyn DatahostStorageTransaction,
        path: &Path,
    ) -> Result<bool, DatahostStorageError>;
    async fn load_option_path_state(
        &self,
        transaction: &mut dyn DatahostStorageTransaction,
        path: &Path,
    ) -> Result<Option<PathState>, DatahostStorageError>;
    async fn load_path_state(
        &self,
        transaction: &mut dyn DatahostStorageTransaction,
        path: &Path,
    ) -> Result<PathState, DatahostStorageError> {
        self.load_option_path_state(transaction, path)
            .await?
            .ok_or_else(|| DatahostStorageError::PathNotFound(path.clone()))
    }
    async fn insert_path_state(
        &self,
        transaction: &mut dyn DatahostStorageTransaction,
        path_state: &PathState,
    ) -> Result<(), DatahostStorageError>;
    async fn update_path_state(
        &self,
        transaction: &mut dyn DatahostStorageTransaction,
        path_state: &PathState,
    ) -> Result<(), DatahostStorageError>;
    async fn delete_path_state(
        &self,
        transaction: &mut dyn DatahostStorageTransaction,
        path: &Path,
    ) -> Result<(), DatahostStorageError>;
}
