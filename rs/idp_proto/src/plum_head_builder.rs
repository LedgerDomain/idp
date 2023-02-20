use crate::{Id, Nonce, PlumBodySeal, PlumHead, PlumRelations, PlumRelationsSeal, UnixNanoseconds};
use anyhow::Result;

#[derive(Default)]
pub struct PlumHeadBuilder {
    plum_head_nonce_o: Option<Nonce>,
    plum_relations_seal_o: Option<PlumRelationsSeal>,
    plum_body_seal_o: Option<PlumBodySeal>,
    owner_id_o: Option<Id>,
    created_at_o: Option<UnixNanoseconds>,
    metadata_o: Option<Vec<u8>>,
}

impl PlumHeadBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    /// Attempts to build a PlumHead, verifying the field values before returning.
    pub fn build(self) -> Result<PlumHead> {
        // Validate attributes
        anyhow::ensure!(self.plum_body_seal_o.is_some(), "PlumHeadBuilder::build can't proceed unless with_plum_body_seal was used to specify the PlumBodySeal");

        Ok(PlumHead {
            plum_head_nonce_o: self.plum_head_nonce_o,
            plum_relations_seal_o: self.plum_relations_seal_o,
            plum_body_seal: self.plum_body_seal_o.unwrap(),
            owner_id_o: self.owner_id_o,
            created_at_o: self.created_at_o,
            metadata_o: self.metadata_o,
        })
    }

    /// Specifies the plum_head_nonce_o field directly.
    pub fn with_plum_head_nonce(mut self, plum_head_nonce: Nonce) -> Self {
        self.plum_head_nonce_o = Some(plum_head_nonce);
        self
    }
    /// Specifies the plum_relations_seal_o field directly.
    pub fn with_plum_relations_seal(mut self, plum_relations_seal: PlumRelationsSeal) -> Self {
        self.plum_relations_seal_o = Some(plum_relations_seal);
        self
    }
    /// Derives the plum_relations_seal_o field from a PlumRelations.  This is a convenience builder
    /// method for when you have the full PlumRelations available.
    pub fn with_relations(self, plum_relations: &PlumRelations) -> Self {
        self.with_plum_relations_seal(PlumRelationsSeal::from(plum_relations))
    }
    // /// Derives the plum_body_seal and plum_body_content_length fields from a PlumBody.  This is a convenience
    // /// builder method for when you have the full PlumBody available.
    // pub fn with_plum_body(self, plum_body: &PlumBody) -> Self {
    //     self.with_plum_body_seal(PlumBodySeal::from(plum_body))
    //         .with_plum_body_content_length(plum_body.plum_body_content.len() as u64)
    // }
    /// Specifies the PlumBodySeal directly.  The body_seal field must be defined in order
    /// to build a well-defined PlumHead.
    pub fn with_plum_body_seal(mut self, plum_body_seal: PlumBodySeal) -> Self {
        self.plum_body_seal_o = Some(plum_body_seal);
        self
    }
    /// Specifies the owner_id_o field directly.
    pub fn with_owner_id(mut self, owner_id: Id) -> Self {
        self.owner_id_o = Some(owner_id);
        self
    }
    /// Specifies the created_at_o field directly.
    pub fn with_created_at(mut self, created_at: UnixNanoseconds) -> Self {
        self.created_at_o = Some(created_at);
        self
    }
    /// Specifies the metadata_o field directly.
    pub fn with_metadata(mut self, metadata: Vec<u8>) -> Self {
        self.metadata_o = Some(metadata);
        self
    }
}
