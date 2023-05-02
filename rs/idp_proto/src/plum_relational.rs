use crate::{PlumHeadSeal, PlumRelationFlags};
use std::collections::HashMap;

/// This trait defines how to derive plum_relations for a given type.
pub trait PlumRelational {
    fn accumulate_plum_relations_nonrecursive(
        &self,
        plum_relation_flags_m: &mut HashMap<PlumHeadSeal, PlumRelationFlags>,
    );
}

impl PlumRelational for str {
    fn accumulate_plum_relations_nonrecursive(
        &self,
        _plum_relation_flags_m: &mut HashMap<PlumHeadSeal, PlumRelationFlags>,
    ) {
        // There are no relations because str has no assumed internal structure.
    }
}

impl PlumRelational for &str {
    fn accumulate_plum_relations_nonrecursive(
        &self,
        _plum_relation_flags_m: &mut HashMap<PlumHeadSeal, PlumRelationFlags>,
    ) {
        // There are no relations because str has no assumed internal structure.
    }
}

impl PlumRelational for String {
    fn accumulate_plum_relations_nonrecursive(
        &self,
        _plum_relation_flags_m: &mut HashMap<PlumHeadSeal, PlumRelationFlags>,
    ) {
        // There are no relations because String has no assumed internal structure.
    }
}

impl PlumRelational for [u8] {
    fn accumulate_plum_relations_nonrecursive(
        &self,
        _plum_relation_flags_m: &mut HashMap<PlumHeadSeal, PlumRelationFlags>,
    ) {
        // There are no relations because [u8] has no assumed internal structure.
    }
}

impl PlumRelational for &[u8] {
    fn accumulate_plum_relations_nonrecursive(
        &self,
        _plum_relation_flags_m: &mut HashMap<PlumHeadSeal, PlumRelationFlags>,
    ) {
        // There are no relations because [u8] has no assumed internal structure.
    }
}

impl PlumRelational for Vec<u8> {
    fn accumulate_plum_relations_nonrecursive(
        &self,
        _plum_relation_flags_m: &mut HashMap<PlumHeadSeal, PlumRelationFlags>,
    ) {
        // There are no relations because Vec<u8> has no assumed internal structure.
    }
}
