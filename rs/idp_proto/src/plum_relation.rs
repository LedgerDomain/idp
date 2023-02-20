use crate::{PlumHeadSeal, PlumRelation, PlumRelationFlagsRaw};
use std::collections::HashMap;

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

// TODO: Ideally this type would be used directly by the code generated from idp.proto, instead
// of using this lame intermediate PlumRelationFlagsRaw type and converting.
bitflags::bitflags! {
    #[derive(serde::Deserialize, serde::Serialize)]
    pub struct PlumRelationFlags: u32 {
        const CONTENT_DEPENDENCY    = 1u32 << (PlumRelation::ContentDependency as u32);
        const METADATA_DEPENDENCY   = 1u32 << (PlumRelation::MetadataDependency as u32);

        const NONE                  = 0;
        const ALL                   = Self::CONTENT_DEPENDENCY.bits | Self::METADATA_DEPENDENCY.bits;
    }
}

/// Convert a single PlumRelation into its PlumRelationFlags counterpart.  The reverse is not
/// possible in general.
impl std::convert::From<PlumRelation> for PlumRelationFlags {
    fn from(plum_relation: PlumRelation) -> Self {
        PlumRelationFlags {
            bits: 1u32 << (plum_relation as u32),
        }
    }
}

/// Convert from the lame PlumRelationFlagsRaw type.  If it's possible to use PlumRelationFlags directly
/// in the generated idp.proto code, then this wouldn't be necessary.
impl std::convert::TryFrom<PlumRelationFlagsRaw> for PlumRelationFlags {
    type Error = anyhow::Error;
    fn try_from(plum_relation_flags_raw: PlumRelationFlagsRaw) -> Result<Self, Self::Error> {
        if plum_relation_flags_raw.value & !(PlumRelationFlags::ALL.bits as u32) != 0 {
            return Err(anyhow::format_err!(
                "PlumRelationFlagsRaw value {:x} out of range (full bitmask is {:x})",
                plum_relation_flags_raw.value,
                PlumRelationFlags::ALL.bits as u32
            ));
        }
        Ok(PlumRelationFlags {
            bits: plum_relation_flags_raw.value,
        })
    }
}

/// Convert into the lame PlumRelationFlagsRaw type.  If it's possible to use PlumRelationFlags directly
/// in the generated idp.proto code, then this wouldn't be necessary.
impl From<PlumRelationFlags> for PlumRelationFlagsRaw {
    fn from(plum_relation_flags: PlumRelationFlags) -> Self {
        Self {
            value: plum_relation_flags.bits,
        }
    }
}

/// This trait defines how to derive plum_relations for a given type.
pub trait PlumRelational {
    fn accumulate_plum_relations_nonrecursive(
        &self,
        plum_relation_flags_m: &mut HashMap<PlumHeadSeal, PlumRelationFlags>,
    );
}

impl std::convert::TryFrom<u32> for PlumRelationFlags {
    type Error = anyhow::Error;
    fn try_from(plum_relation_flags_raw: u32) -> Result<Self, Self::Error> {
        if plum_relation_flags_raw > PlumRelationFlags::ALL.bits {
            return Err(anyhow::format_err!(
                "invalid PlumRelationFlags value {:x}; expected a value in the range [0, {:x}]",
                plum_relation_flags_raw,
                PlumRelationFlags::ALL.bits
            ));
        }
        Ok(PlumRelationFlags {
            bits: plum_relation_flags_raw,
        })
    }
}
