use crate::{ContentMetadata, PlumBodySeal, PlumHeadSeal, PlumMetadataSeal, PlumRelationsSeal};

#[derive(Clone, Debug, thiserror::Error)]
pub enum PlumVerifyError {
    #[error("computed PlumMetadataSeal was {computed_plum_metadata_seal} but value represented in PlumHead was {expected_plum_metadata_seal}")]
    ComputedPlumMetadataSealMismatch {
        computed_plum_metadata_seal: PlumMetadataSeal,
        expected_plum_metadata_seal: PlumMetadataSeal,
    },
    #[error("computed PlumRelationsSeal was {computed_plum_relations_seal} but value represented in PlumHead was {expected_plum_relations_seal}")]
    ComputedPlumRelationsSealMismatch {
        computed_plum_relations_seal: PlumRelationsSeal,
        expected_plum_relations_seal: PlumRelationsSeal,
    },
    #[error("computed PlumBodySeal was {computed_plum_body_seal} but value represented in PlumHead was {expected_plum_body_seal}")]
    ComputedPlumBodySealMismatch {
        computed_plum_body_seal: PlumBodySeal,
        expected_plum_body_seal: PlumBodySeal,
    },
    #[error("plum_body_seal in PlumHead was {plum_head_plum_body_seal} but source_plum_body_seal in PlumRelations was {plum_relations_source_plum_body_seal}")]
    PlumBodySealRedundancyMismatch {
        plum_head_plum_body_seal: PlumBodySeal,
        plum_relations_source_plum_body_seal: PlumBodySeal,
    },
    #[error("Plum {plum_head_seal} had a mismatch between plum_body_content_metadata in PlumMetadata which was {plum_metadata_plum_body_content_metadata:?} while plum_body_content.metadata in PlumBody was {plum_body_plum_body_content_metadata:?}")]
    PlumBodyContentMetadataRedundancyMismatch {
        plum_head_seal: PlumHeadSeal,
        plum_metadata_plum_body_content_metadata: ContentMetadata,
        plum_body_plum_body_content_metadata: ContentMetadata,
    },
}
