use crate::{Relational, RelationFlags};
use idp_proto::PlumHeadSeal;
use std::collections::{BTreeMap, HashMap};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct DirNode {
    /// Ordered map of entry -> PlumHeadSeal.  Analogous to filenames mapping to INode values in a filesystem.
    pub entry_m: BTreeMap<String, PlumHeadSeal>,
}

impl Relational for DirNode {
    fn accumulate_relations_nonrecursive(
        &self,
        relation_m: &mut HashMap<PlumHeadSeal, RelationFlags>,
        mask: RelationFlags,
    ) -> Result<(), failure::Error> {
        if mask & RelationFlags::CONTENT_DEPENDENCY != RelationFlags::NONE {
            // Only bother if the mask includes CONTENT_DEPENDENCY, because that's all that's in DirNode.
            for entry in self.entry_m.values() {
                match relation_m.get_mut(&entry) {
                    Some(relation_flags) => { *relation_flags |= RelationFlags::CONTENT_DEPENDENCY; }
                    None => { relation_m.insert(entry.clone(), RelationFlags::CONTENT_DEPENDENCY); }
                }
            }
        }
        Ok(())
    }
}
