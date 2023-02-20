use crate::{DatahostStorageError, DatahostStorageTransaction};
use idp_proto::{
    Plum, PlumBody, PlumBodySeal, PlumHead, PlumHeadSeal, PlumRelations, PlumRelationsSeal,
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
        if let Some(plum_relations_seal) = plum_head.plum_relations_seal_o.as_ref() {
            if !self
                .has_plum_relations(transaction, plum_relations_seal)
                .await?
            {
                return Ok(false);
            }
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
        let computed_plum_relations_seal_o =
            if let Some(plum_relations) = plum.plum_relations_o.as_ref() {
                Some(
                    self.store_plum_relations(transaction, plum_relations)
                        .await?,
                )
            } else {
                None
            };
        plum.plum_head
            .verify_plum_relations_seal_o(computed_plum_relations_seal_o.as_ref())?;

        let computed_plum_body_seal = self.store_plum_body(transaction, &plum.plum_body).await?;
        plum.plum_head
            .verify_plum_body_seal(&computed_plum_body_seal)?;

        let plum_head_seal = self.store_plum_head(transaction, &plum.plum_head).await?;

        Ok(plum_head_seal)
    }

    async fn load_option_plum_head(
        &self,
        transaction: &mut dyn DatahostStorageTransaction,
        plum_head_seal: &PlumHeadSeal,
    ) -> Result<Option<PlumHead>, DatahostStorageError>;
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
    /// If any of the expected components (PlumHead, PlumBody, or optional PlumRelations) are missing, then this returns
    /// None.  Otherwise returns Some(plum), where plum is the Plum with those components.
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

        let plum_relations_o = match &plum_head.plum_relations_seal_o {
            Some(plum_relations_seal) => {
                let plum_relations = self
                    .load_plum_relations(transaction, plum_relations_seal)
                    .await?;
                Some(plum_relations)
            }
            None => None,
        };
        // Sanity check -- should be true by the verification upon store operation, but it's possible that the DB is corrupted.
        plum_head
            .verify_plum_relations_seal_o(
                plum_relations_o
                    .as_ref()
                    .map(PlumRelationsSeal::from)
                    .as_ref(),
            )
            .expect("programmer error or corrupted DB; PlumRelationsSeal did not verify");

        let plum_body_o = self
            .load_option_plum_body(transaction, &plum_head.plum_body_seal)
            .await?;
        if plum_body_o.is_none() {
            return Ok(None);
        }
        let plum_body = plum_body_o.unwrap();
        // Sanity check -- should be true by the verification upon store operation, but it's possible that the DB is corrupted.
        plum_head
            .verify_plum_body_seal(&PlumBodySeal::from(&plum_body))
            .expect("programmer error or corrupted DB; PlumRelationsSeal did not verify");

        Ok(Some(Plum {
            plum_head,
            plum_relations_o,
            plum_body,
        }))
    }
    async fn load_plum(
        &self,
        transaction: &mut dyn DatahostStorageTransaction,
        plum_head_seal: &PlumHeadSeal,
    ) -> Result<Plum, DatahostStorageError> {
        let plum_head = self.load_plum_head(transaction, plum_head_seal).await?;

        let plum_relations_o = match &plum_head.plum_relations_seal_o {
            Some(plum_relations_seal) => {
                let plum_relations = self
                    .load_plum_relations(transaction, plum_relations_seal)
                    .await?;
                Some(plum_relations)
            }
            None => None,
        };
        // Sanity check -- should be true by the verification upon store operation, but it's possible that the DB is corrupted.
        plum_head
            .verify_plum_relations_seal_o(
                plum_relations_o
                    .as_ref()
                    .map(PlumRelationsSeal::from)
                    .as_ref(),
            )
            .expect("programmer error or corrupted DB; PlumRelationsSeal did not verify");

        let plum_body = self
            .load_plum_body(transaction, &plum_head.plum_body_seal)
            .await?;
        // Sanity check -- should be true by the verification upon store operation, but it's possible that the DB is corrupted.
        plum_head
            .verify_plum_body_seal(&PlumBodySeal::from(&plum_body))
            .expect("programmer error or corrupted DB; PlumRelationsSeal did not verify");

        Ok(Plum {
            plum_head,
            plum_relations_o,
            plum_body,
        })
    }
}
