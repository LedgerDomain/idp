use crate::{Plum, PlumBody, PlumRelations, PlumVerifyError};

impl Plum {
    /// This verifies the PlumRelationsSeal (if specified in plum_head) and PlumBodySeal
    /// relative to those in the PlumHead.
    pub fn verify(&self) -> Result<(), PlumVerifyError> {
        let computed_plum_relations_seal_o =
            self.plum_relations_o.as_ref().map(PlumRelationsSeal::from);
        let computed_plum_body_seal = PlumBodySeal::from(&self.plum_body);
        self.plum_head
            .verify(computed_plum_relations_seal_o, computed_plum_body_seal)
    }
}
