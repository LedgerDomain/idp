use crate::{FragmentQueryResult, FragmentQueryable};
use anyhow::Result;
use idp_proto::{PlumHeadSeal, PlumRelationFlags};
use std::collections::{BTreeMap, HashMap};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct DirNode {
    /// Ordered map of entry -> PlumHeadSeal.  Analogous to filenames mapping to INode values in a filesystem.
    pub entry_m: BTreeMap<String, PlumHeadSeal>,
}

impl idp_proto::ContentClassifiable for DirNode {
    fn content_class_str() -> &'static str {
        "application/x.idp.DirNode"
    }
    fn derive_content_class_str(&self) -> &'static str {
        Self::content_class_str()
    }
    fn default_content_format(&self) -> Option<idp_proto::ContentFormat> {
        None
    }
    fn validate_content_format(&self, content_format: &idp_proto::ContentFormat) -> Result<()> {
        idp_proto::validate_is_serde_format(content_format)
    }
}

impl idp_proto::Deserializable for DirNode {
    fn deserialize_using_format(
        content_format: &idp_proto::ContentFormat,
        reader: &mut dyn std::io::Read,
    ) -> Result<Self> {
        idp_proto::deserialize_using_serde_format(content_format, reader)
    }
}

impl idp_proto::Serializable for DirNode {
    fn serialize_using_format(
        &self,
        content_format: &idp_proto::ContentFormat,
        writer: &mut dyn std::io::Write,
    ) -> Result<()> {
        idp_proto::serialize_using_serde_format(self, content_format, writer)
    }
}

impl idp_proto::PlumRelational for DirNode {
    fn accumulate_plum_relations_nonrecursive(
        &self,
        plum_relation_flags_m: &mut HashMap<PlumHeadSeal, PlumRelationFlags>,
    ) {
        for entry in self.entry_m.values() {
            match plum_relation_flags_m.get_mut(&entry) {
                Some(plum_relation_flags) => {
                    *plum_relation_flags |= PlumRelationFlags::CONTENT_DEPENDENCY;
                }
                None => {
                    plum_relation_flags_m
                        .insert(entry.clone(), PlumRelationFlags::CONTENT_DEPENDENCY);
                }
            }
        }
    }
}

impl<'a> FragmentQueryable<'a> for DirNode {
    /// For DirNode, the query_str should have one of the following forms:
    ///     0.  <empty-string>
    ///     1.  <entry-name>
    ///     2.  <entry-name>/<rest-of-query-str>
    /// In case 0, the DirNode itself will be returned (as its PlumHeadSeal).
    /// In case 1, the PlumHeadSeal of the entry will be returned.
    /// In case 2, <rest-of-query-str> will be forwarded to query the Plum referred to by <entry-name>.
    /// TODO: Add other things like:
    /// -   Number of entries
    /// -   Recursive sum of content
    fn fragment_query_single_segment(
        &self,
        self_plum_head_seal: &PlumHeadSeal,
        query_str: &'a str,
    ) -> Result<FragmentQueryResult<'a>> {
        // If query_str is empty, return this DirNode's PlumHeadSeal.
        if query_str.is_empty() {
            return Ok(FragmentQueryResult::Value(self_plum_head_seal.clone()));
        }
        let (entry_name, rest_of_query_str_o) = match query_str.split_once('/') {
            Some((entry_name, rest_of_query_str)) => (entry_name, Some(rest_of_query_str)),
            None => (query_str, None),
        };
        // Have to handle empty again, since a query_str of "/" will cause entry_name to be empty.
        if entry_name.is_empty() {
            return Ok(FragmentQueryResult::Value(self_plum_head_seal.clone()));
        }
        let entry = match self.entry_m.get(entry_name) {
            Some(entry) => entry,
            None => {
                return Err(anyhow::format_err!(
                    "DirNode {} did not contain entry {:?}",
                    self_plum_head_seal,
                    entry_name
                ));
            }
        };
        match rest_of_query_str_o {
            Some(rest_of_query_str) => Ok(FragmentQueryResult::ForwardQueryTo {
                target: entry.clone(),
                rest_of_query_str,
            }),
            None => Ok(FragmentQueryResult::Value(entry.clone())),
        }
    }
}
