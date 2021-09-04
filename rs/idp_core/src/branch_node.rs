use crate::{Relational, RelationFlags};
use idp_proto::PlumHeadSeal;
use std::collections::HashMap;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct BranchNode {
    /// This is the direct ancestor of this BranchNode.  It must refer to a BranchNode itself.
    pub ancestor_o: Option<PlumHeadSeal>,
    /// This could have any type, but should probably adhere to some general metadata schema.
    pub metadata: PlumHeadSeal,
    /// This is the actual content of this BranchNode.  TODO: Should there be a delta attribute separate from content?
    pub content_o: Option<PlumHeadSeal>,
    /// This specifies the diff from the previous state of this branch to this state.
    pub posi_diff_o: Option<PlumHeadSeal>,
    /// This specifies the diff from this state of this branch to the previous state.
    pub nega_diff_o: Option<PlumHeadSeal>,
    // TODO: merge nodes
}

impl Relational for BranchNode {
    fn accumulate_relations_nonrecursive(
        &self,
        relation_m: &mut HashMap<PlumHeadSeal, RelationFlags>,
        mask: RelationFlags,
    ) -> Result<(), failure::Error> {
        if mask & RelationFlags::METADATA_DEPENDENCY != RelationFlags::NONE {
            if let Some(ancestor) = &self.ancestor_o {
                match relation_m.get_mut(&ancestor) {
                    Some(relation_flags) => { *relation_flags |= RelationFlags::METADATA_DEPENDENCY; }
                    None => { relation_m.insert(ancestor.clone(), RelationFlags::METADATA_DEPENDENCY); }
                }
            }
            match relation_m.get_mut(&self.metadata) {
                Some(relation_flags) => { *relation_flags |= RelationFlags::METADATA_DEPENDENCY; }
                None => { relation_m.insert(self.metadata.clone(), RelationFlags::METADATA_DEPENDENCY); }
            }
        }
        if mask & RelationFlags::CONTENT_DEPENDENCY != RelationFlags::NONE {
            if let Some(content) = &self.content_o {
                match relation_m.get_mut(&content) {
                    Some(relation_flags) => { *relation_flags |= RelationFlags::CONTENT_DEPENDENCY; }
                    None => { relation_m.insert(content.clone(), RelationFlags::CONTENT_DEPENDENCY); }
                }
            }
            if let Some(posi_diff) = &self.posi_diff_o {
                match relation_m.get_mut(&posi_diff) {
                    Some(relation_flags) => { *relation_flags |= RelationFlags::CONTENT_DEPENDENCY; }
                    None => { relation_m.insert(posi_diff.clone(), RelationFlags::CONTENT_DEPENDENCY); }
                }
            }
            if let Some(nega_diff) = &self.nega_diff_o {
                match relation_m.get_mut(&nega_diff) {
                    Some(relation_flags) => { *relation_flags |= RelationFlags::CONTENT_DEPENDENCY; }
                    None => { relation_m.insert(nega_diff.clone(), RelationFlags::CONTENT_DEPENDENCY); }
                }
            }
        }
        Ok(())
    }
}
