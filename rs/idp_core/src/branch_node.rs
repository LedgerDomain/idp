use crate::{FragmentQueryResult, FragmentQueryable};
use anyhow::Result;
use idp_proto::{PlumHeadSeal, PlumRelationFlags};
use std::collections::HashMap;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct BranchNode {
    /// This is the direct ancestor of this BranchNode.  It must refer to a BranchNode Plum whose
    /// height is less than that of this BranchNode.
    pub ancestor_o: Option<PlumHeadSeal>,
    /// The height gives a useful way to rule out certain causal orders when comparing BranchNode-s.
    /// The height of a BranchNode must be greater than the max of the heights of this node's ancestor(s).
    /// By convention, if there are no ancestors, the height is defined to be 0.  Note that this is
    /// only required to be a local property of the BranchNode DAG, not a global property.
    pub height: u64,
    /// The Plum this refers to could have any type, but should probably adhere to some general metadata schema.
    pub metadata: PlumHeadSeal,
    /// This is the actual content Plum of this BranchNode.
    // TODO: Should this not be non-optional? The argument for optional is that the actual state of the
    // branch can be tracked separately, using the posi- and nega-diffs.  Maybe there can be a separate
    // version of BranchNode that has these semantics.
    pub content_o: Option<PlumHeadSeal>,
    /// This specifies the diff from the previous state of this branch to this state.
    pub posi_diff_o: Option<PlumHeadSeal>,
    /// This specifies the diff from this state of this branch to the previous state.
    pub nega_diff_o: Option<PlumHeadSeal>,
    // TODO: merge nodes (or generally an arbitrary number of ancestors)
}

impl idp_proto::ContentClassifiable for BranchNode {
    fn content_class_str() -> &'static str {
        "application/x.idp.BranchNode"
    }
    fn derive_content_class_str(&self) -> &'static str {
        Self::content_class_str()
    }
}

impl idp_proto::Deserializable for BranchNode {
    fn deserialize_using_format(
        content_format: &idp_proto::ContentFormat,
        reader: &mut dyn std::io::Read,
    ) -> Result<Self> {
        idp_proto::deserialize_using_serde_format(content_format, reader)
    }
}

impl idp_proto::Serializable for BranchNode {
    fn serialize_using_format(
        &self,
        content_format: &idp_proto::ContentFormat,
        writer: &mut dyn std::io::Write,
    ) -> Result<()> {
        idp_proto::serialize_using_serde_format(self, content_format, writer)
    }
}

impl idp_proto::PlumRelational for BranchNode {
    fn accumulate_plum_relations_nonrecursive(
        &self,
        plum_relation_flags_m: &mut HashMap<PlumHeadSeal, PlumRelationFlags>,
    ) {
        if let Some(ancestor) = &self.ancestor_o {
            match plum_relation_flags_m.get_mut(&ancestor) {
                Some(plum_relation_flags) => {
                    *plum_relation_flags |= PlumRelationFlags::METADATA_DEPENDENCY;
                }
                None => {
                    plum_relation_flags_m
                        .insert(ancestor.clone(), PlumRelationFlags::METADATA_DEPENDENCY);
                }
            }
        }
        match plum_relation_flags_m.get_mut(&self.metadata) {
            Some(plum_relation_flags) => {
                *plum_relation_flags |= PlumRelationFlags::METADATA_DEPENDENCY;
            }
            None => {
                plum_relation_flags_m.insert(
                    self.metadata.clone(),
                    PlumRelationFlags::METADATA_DEPENDENCY,
                );
            }
        }
        if let Some(content) = &self.content_o {
            match plum_relation_flags_m.get_mut(&content) {
                Some(plum_relation_flags) => {
                    *plum_relation_flags |= PlumRelationFlags::CONTENT_DEPENDENCY;
                }
                None => {
                    plum_relation_flags_m
                        .insert(content.clone(), PlumRelationFlags::CONTENT_DEPENDENCY);
                }
            }
        }
        // TODO: This may call for a different kind of dependency, since the diffs aren't primary data,
        // and in principle can be derived from this and its direct ancestor.
        if let Some(posi_diff) = &self.posi_diff_o {
            match plum_relation_flags_m.get_mut(&posi_diff) {
                Some(plum_relation_flags) => {
                    *plum_relation_flags |= PlumRelationFlags::CONTENT_DEPENDENCY;
                }
                None => {
                    plum_relation_flags_m
                        .insert(posi_diff.clone(), PlumRelationFlags::CONTENT_DEPENDENCY);
                }
            }
        }
        // TODO: This may call for a different kind of dependency, since the diffs aren't primary data,
        // and in principle can be derived from this and its direct ancestor.
        if let Some(nega_diff) = &self.nega_diff_o {
            match plum_relation_flags_m.get_mut(&nega_diff) {
                Some(plum_relation_flags) => {
                    *plum_relation_flags |= PlumRelationFlags::CONTENT_DEPENDENCY;
                }
                None => {
                    plum_relation_flags_m
                        .insert(nega_diff.clone(), PlumRelationFlags::CONTENT_DEPENDENCY);
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
