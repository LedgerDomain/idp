use crate::PlumRelation;

impl std::convert::TryFrom<i32> for PlumRelation {
    type Error = anyhow::Error;
    fn try_from(relation_raw: i32) -> Result<Self, Self::Error> {
        let plum_relation =
            match <PlumRelation as num_traits::FromPrimitive>::from_i32(relation_raw) {
                Some(plum_relation) => plum_relation,
                None => {
                    let lowest_raw = PlumRelation::ContentDependency as i32;
                    // NOTE: This must be updated if/when enum variants are added to PlumRelation in idp.proto
                    let highest_raw = PlumRelation::MetadataDependency as i32;
                    return Err(anyhow::format_err!(
                        "invalid PlumRelation value {}; expected a value in the range [{}, {}]",
                        relation_raw,
                        lowest_raw,
                        highest_raw
                    ));
                }
            };
        Ok(plum_relation)
    }
}

// pub enum PlumRelation {
//     ContentDependency   = 0,
//     MetadataDependency  = 1,
// }
