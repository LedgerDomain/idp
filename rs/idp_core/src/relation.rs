use idp_proto::PlumHeadSeal;
use std::collections::HashMap;

pub enum Relation {
    ContentDependency   = 0,
    MetadataDependency  = 1,
}

bitflags::bitflags! {
    pub struct RelationFlags: u32 {
        const CONTENT_DEPENDENCY    = 1u32 << (Relation::ContentDependency as u32);
        const METADATA_DEPENDENCY   = 1u32 << (Relation::MetadataDependency as u32);

        const NONE                  = 0;
        const ALL                   = Self::CONTENT_DEPENDENCY.bits | Self::METADATA_DEPENDENCY.bits;
    }
}

pub trait Relational {
    fn accumulate_relations_nonrecursive(
        &self,
        relation_m: &mut HashMap<PlumHeadSeal, RelationFlags>,
        mask: RelationFlags,
    ) -> Result<(), failure::Error>;
}
