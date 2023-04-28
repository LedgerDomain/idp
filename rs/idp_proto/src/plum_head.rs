use crate::{
    Hashable, PlumBodySeal, PlumHead, PlumMetadataSeal, PlumRelationsSeal, PlumVerifyError,
};

impl PlumHead {
    pub fn verify_plum_metadata_seal(
        &self,
        computed_plum_metadata_seal: &PlumMetadataSeal,
    ) -> Result<(), PlumVerifyError> {
        if *computed_plum_metadata_seal != self.plum_metadata_seal {
            return Err(PlumVerifyError::ComputedPlumMetadataSealMismatch {
                computed_plum_metadata_seal: computed_plum_metadata_seal.clone(),
                expected_plum_metadata_seal: self.plum_metadata_seal.clone(),
            });
        }
        Ok(())
    }
    pub fn verify_plum_relations_seal(
        &self,
        computed_plum_relations_seal: &PlumRelationsSeal,
    ) -> Result<(), PlumVerifyError> {
        if *computed_plum_relations_seal != self.plum_relations_seal {
            return Err(PlumVerifyError::ComputedPlumRelationsSealMismatch {
                computed_plum_relations_seal: computed_plum_relations_seal.clone(),
                expected_plum_relations_seal: self.plum_relations_seal.clone(),
            });
        }
        Ok(())
    }
    pub fn verify_plum_body_seal(
        &self,
        computed_plum_body_seal: &PlumBodySeal,
    ) -> Result<(), PlumVerifyError> {
        if *computed_plum_body_seal != self.plum_body_seal {
            return Err(PlumVerifyError::ComputedPlumBodySealMismatch {
                computed_plum_body_seal: computed_plum_body_seal.clone(),
                expected_plum_body_seal: self.plum_body_seal.clone(),
            });
        }
        Ok(())
    }
    /// This only verifies that the seals of this PlumHead match the computed seals.
    pub fn verify_seals(
        &self,
        computed_plum_metadata_seal: &PlumMetadataSeal,
        computed_plum_relations_seal: &PlumRelationsSeal,
        computed_plum_body_seal: &PlumBodySeal,
    ) -> Result<(), PlumVerifyError> {
        self.verify_plum_metadata_seal(computed_plum_metadata_seal)?;
        self.verify_plum_relations_seal(computed_plum_relations_seal)?;
        self.verify_plum_body_seal(computed_plum_body_seal)?;
        Ok(())
    }
}

impl Hashable for PlumHead {
    fn update_hasher(&self, hasher: &mut sha2::Sha256) {
        // NOTE: The specific order and form of this hashing must NOT be changed!
        self.plum_head_nonce_o.update_hasher(hasher);
        self.plum_metadata_seal.update_hasher(hasher);
        self.plum_relations_seal.update_hasher(hasher);
        self.plum_body_seal.update_hasher(hasher);
    }
}
