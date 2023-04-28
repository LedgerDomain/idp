use crate::{
    Plum, PlumBodySeal, PlumHeadSeal, PlumMetadataSeal, PlumRelationsSeal, PlumVerifyError,
};

impl Plum {
    /// Verify that the seals in the PlumHead match the computed seals of its components,
    /// and verify all other constraints between the components.
    pub fn verify(&self) -> Result<(), PlumVerifyError> {
        //
        // First, compute the seals from this Plum's actual content, and verify the seals.
        //

        let computed_plum_metadata_seal = PlumMetadataSeal::from(&self.plum_metadata);
        let computed_plum_relations_seal = PlumRelationsSeal::from(&self.plum_relations);
        let computed_plum_body_seal = PlumBodySeal::from(&self.plum_body);
        self.plum_head.verify_seals(
            &computed_plum_metadata_seal,
            &computed_plum_relations_seal,
            &computed_plum_body_seal,
        )?;

        //
        // Then verify all the various higher-order constraints between the components.
        //

        // The PlumRelations source_plum_body_seal must match the PlumHead plum_body_seal.
        if self.plum_relations.source_plum_body_seal != self.plum_head.plum_body_seal {
            return Err(PlumVerifyError::PlumBodySealRedundancyMismatch {
                plum_head_plum_body_seal: self.plum_head.plum_body_seal.clone(),
                plum_relations_source_plum_body_seal: self
                    .plum_relations
                    .source_plum_body_seal
                    .clone(),
            });
        }
        // If PlumMetadata contains a PlumBodyContentMetadata, then it must match that of the PlumBody.
        if let Some(plum_metadata_plum_body_content_metadata) =
            self.plum_metadata.plum_body_content_metadata_o.as_ref()
        {
            if *plum_metadata_plum_body_content_metadata
                != self.plum_body.plum_body_content.content_metadata
            {
                let plum_head_seal = PlumHeadSeal::from(self);
                return Err(PlumVerifyError::PlumBodyContentMetadataRedundancyMismatch {
                    plum_head_seal,
                    plum_metadata_plum_body_content_metadata:
                        plum_metadata_plum_body_content_metadata.clone(),
                    plum_body_plum_body_content_metadata: self
                        .plum_body
                        .plum_body_content
                        .content_metadata
                        .clone(),
                });
            }
        }

        Ok(())
    }
}
