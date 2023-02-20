use crate::{PlumBodySeal, PlumRelationsSeal};

#[derive(Clone, Debug, thiserror::Error)]
pub enum PlumVerifyError {
    #[error("PlumHead expected plum_relations {0}, but encountered no plum_relations")]
    ExpectedPlumRelations(PlumRelationsSeal),
    #[error("PlumHead expected no plum_relations, but encountered plum_relations {0}")]
    UnexpectedPlumRelations(PlumRelationsSeal),
    #[error("computed PlumRelationsSeal was {computed_plum_relations_seal} but value represented in PlumHead was {expected_plum_relations_seal}")]
    PlumRelationsSealMismatch {
        computed_plum_relations_seal: PlumRelationsSeal,
        expected_plum_relations_seal: PlumRelationsSeal,
    },
    #[error("computed PlumBodySeal was {computed_plum_body_seal} but value represented in PlumHead was {expected_plum_body_seal}")]
    PlumBodySealMismatch {
        computed_plum_body_seal: PlumBodySeal,
        expected_plum_body_seal: PlumBodySeal,
    },
}
