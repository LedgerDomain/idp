use crate::{FragmentQueryResult, FragmentQueryable};
use anyhow::Result;
use idp_proto::{ContentType, ContentTypeable, PlumHeadSeal, RelationFlags, Relational};
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

impl ContentTypeable for BranchNode {
    fn content_type() -> ContentType {
        ContentType::from("idp::BranchNode")
    }
}

impl Relational for BranchNode {
    fn accumulate_relations_nonrecursive(
        &self,
        relation_flags_m: &mut HashMap<PlumHeadSeal, RelationFlags>,
    ) {
        if let Some(ancestor) = &self.ancestor_o {
            match relation_flags_m.get_mut(&ancestor) {
                Some(relation_flags) => {
                    *relation_flags |= RelationFlags::METADATA_DEPENDENCY;
                }
                None => {
                    relation_flags_m.insert(ancestor.clone(), RelationFlags::METADATA_DEPENDENCY);
                }
            }
        }
        match relation_flags_m.get_mut(&self.metadata) {
            Some(relation_flags) => {
                *relation_flags |= RelationFlags::METADATA_DEPENDENCY;
            }
            None => {
                relation_flags_m.insert(self.metadata.clone(), RelationFlags::METADATA_DEPENDENCY);
            }
        }
        if let Some(content) = &self.content_o {
            match relation_flags_m.get_mut(&content) {
                Some(relation_flags) => {
                    *relation_flags |= RelationFlags::CONTENT_DEPENDENCY;
                }
                None => {
                    relation_flags_m.insert(content.clone(), RelationFlags::CONTENT_DEPENDENCY);
                }
            }
        }
        if let Some(posi_diff) = &self.posi_diff_o {
            match relation_flags_m.get_mut(&posi_diff) {
                Some(relation_flags) => {
                    *relation_flags |= RelationFlags::CONTENT_DEPENDENCY;
                }
                None => {
                    relation_flags_m.insert(posi_diff.clone(), RelationFlags::CONTENT_DEPENDENCY);
                }
            }
        }
        if let Some(nega_diff) = &self.nega_diff_o {
            match relation_flags_m.get_mut(&nega_diff) {
                Some(relation_flags) => {
                    *relation_flags |= RelationFlags::CONTENT_DEPENDENCY;
                }
                None => {
                    relation_flags_m.insert(nega_diff.clone(), RelationFlags::CONTENT_DEPENDENCY);
                }
            }
        }
    }
}

impl<'a> FragmentQueryable<'a> for BranchNode {
    /// For BranchNode, the query_str should have one of the following forms:
    ///     0.  <empty-string>
    ///     1.  <entry-name>
    ///     2.  <entry-name>/<rest-of-query-str>
    /// where <entry-name> must be one of:
    /// -   ancestor
    /// -   metadata
    /// -   content
    /// TODO: Document the fact that ancestor and content can be None; maybe make a way to query that fact.
    /// TODO: Support posi_diff and nega_diff when they're implemented.
    /// In case 0, the BranchNode itself will be returned (as its PlumHeadSeal).
    /// In case 1, the PlumHeadSeal of the entry will be returned.
    /// In case 2, <rest-of-query-str> will be forwarded to query the Plum referred to by <entry-name>.
    fn fragment_query_single_segment(
        &self,
        self_plum_head_seal: &PlumHeadSeal,
        query_str: &'a str,
    ) -> Result<FragmentQueryResult<'a>> {
        // If query_str is empty, return this BranchNode's PlumHeadSeal.
        if query_str.is_empty() {
            return Ok(FragmentQueryResult::Value(self_plum_head_seal.clone()));
        }
        let (entry_name, rest_of_query_str_o) = match query_str.split_once('/') {
            Some((entry_name, rest_of_query_str)) => (entry_name, Some(rest_of_query_str)),
            None => (query_str, None),
        };
        let entry_o = match entry_name {
            "ancestor" => self.ancestor_o.clone(),
            "metadata" => Some(self.metadata.clone()),
            "content" => self.content_o.clone(),
            _ => {
                return Err(anyhow::format_err!(
                    "BranchNode does not have entry {:?}",
                    entry_name
                ));
            }
        };
        if entry_o.is_none() {
            return Err(anyhow::format_err!(
                "BranchNode entry {} is not set for BranchNode {}",
                entry_name,
                self_plum_head_seal
            ));
        }
        let entry = entry_o.unwrap();
        match rest_of_query_str_o {
            Some(rest_of_query_str) => Ok(FragmentQueryResult::ForwardQueryTo {
                target: entry.clone(),
                rest_of_query_str,
            }),
            None => Ok(FragmentQueryResult::Value(entry.clone())),
        }
    }
}
