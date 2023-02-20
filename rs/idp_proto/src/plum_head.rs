use crate::{PlumBodySeal, PlumHead, PlumRelationsSeal, PlumVerifyError};

impl PlumHead {
    pub fn verify_plum_relations_seal_o(
        &self,
        computed_plum_relations_seal_o: Option<&PlumRelationsSeal>,
    ) -> Result<(), PlumVerifyError> {
        let expected_plum_relations_seal_o = self.plum_relations_seal_o.as_ref();
        match (
            expected_plum_relations_seal_o,
            computed_plum_relations_seal_o,
        ) {
            (Some(expected_plum_relations_seal), Some(computed_plum_relations_seal)) => {
                if *computed_plum_relations_seal == *expected_plum_relations_seal {
                    Ok(())
                } else {
                    Err(PlumVerifyError::PlumRelationsSealMismatch {
                        computed_plum_relations_seal: computed_plum_relations_seal.clone(),
                        expected_plum_relations_seal: expected_plum_relations_seal.clone(),
                    })
                }
            }
            (Some(expected_plum_relations_seal), None) => Err(
                PlumVerifyError::ExpectedPlumRelations(expected_plum_relations_seal.clone()),
            ),
            (None, Some(computed_plum_relations_seal)) => Err(
                PlumVerifyError::UnexpectedPlumRelations(computed_plum_relations_seal.clone()),
            ),
            (None, None) => Ok(()),
        }
    }
    pub fn verify_plum_body_seal(
        &self,
        computed_plum_body_seal: &PlumBodySeal,
    ) -> Result<(), PlumVerifyError> {
        if *computed_plum_body_seal != self.plum_body_seal {
            return Err(PlumVerifyError::PlumBodySealMismatch {
                computed_plum_body_seal: computed_plum_body_seal.clone(),
                expected_plum_body_seal: self.plum_body_seal.clone(),
            });
        }
        Ok(())
    }
    pub fn verify(
        &self,
        computed_plum_relations_seal_o: Option<&PlumRelationsSeal>,
        computed_plum_body_seal: &PlumBodySeal,
    ) -> Result<(), PlumVerifyError> {
        self.verify_plum_relations_seal_o(computed_plum_relations_seal_o)?;
        self.verify_plum_body_seal(computed_plum_body_seal)?;
        Ok(())
    }
}
