use crate::{FragmentQueryable, FragmentQueryResult};
use idp_proto::{ContentType, ContentTypeable, PlumHeadSeal, RelationFlags};
use std::collections::{BTreeMap, HashMap};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct DirNode {
    /// Ordered map of entry -> PlumHeadSeal.  Analogous to filenames mapping to INode values in a filesystem.
    pub entry_m: BTreeMap<String, PlumHeadSeal>,
}

impl ContentTypeable for DirNode {
    fn content_type() -> ContentType {
        ContentType::from("idp::DirNode")
    }
}

impl idp_proto::Relational for DirNode {
    fn accumulate_relations_nonrecursive(
        &self,
        relation_flags_m: &mut HashMap<PlumHeadSeal, RelationFlags>,
    ) {
        for entry in self.entry_m.values() {
            match relation_flags_m.get_mut(&entry) {
                Some(relation_flags) => { *relation_flags |= RelationFlags::CONTENT_DEPENDENCY; }
                None => { relation_flags_m.insert(entry.clone(), RelationFlags::CONTENT_DEPENDENCY); }
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
    ) -> Result<FragmentQueryResult<'a>, failure::Error> {
        // If query_str is empty, return this DirNode's PlumHeadSeal.
        if query_str.is_empty() {
            return Ok(FragmentQueryResult::Value(self_plum_head_seal.clone()));
        }
        let (entry_name, rest_of_query_str_o) = match query_str.split_once('/') {
            Some((entry_name, rest_of_query_str)) => (entry_name, Some(rest_of_query_str)),
            None => (query_str, None)
        };
        // Have to handle empty again, since a query_str of "/" will cause entry_name to be empty.
        if entry_name.is_empty() {
            return Ok(FragmentQueryResult::Value(self_plum_head_seal.clone()));
        }
        let entry = match self.entry_m.get(entry_name) {
            Some(entry) => entry,
            None => {
                return Err(failure::format_err!("DirNode {} did not contain entry {:?}", self_plum_head_seal, entry_name));
            }
        };
        match rest_of_query_str_o {
            Some(rest_of_query_str) => Ok(FragmentQueryResult::ForwardQueryTo { target: entry.clone(), rest_of_query_str }),
            None => Ok(FragmentQueryResult::Value(entry.clone()))
        }
    }
}
