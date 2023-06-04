use crate::Result;
use idp_proto::{PlumHeadSeal, PlumRelationFlags};
use std::collections::HashMap;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct OwnedData {
    pub owner: String,
    pub data: PlumHeadSeal,
    /// Can optionally be used to turn this into a node in a microledger of OwnedData-s.
    pub previous_owned_data_o: Option<PlumHeadSeal>,
}

impl idp_proto::ContentClassifiable for OwnedData {
    fn content_class_str() -> &'static str {
        "application/x.idp.example.sig.OwnedData"
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

impl idp_proto::Deserializable for OwnedData {
    fn deserialize_using_format(
        content_format: &idp_proto::ContentFormat,
        reader: &mut dyn std::io::Read,
    ) -> Result<Self> {
        idp_proto::deserialize_using_serde_format(content_format, reader)
    }
}

impl idp_proto::Serializable for OwnedData {
    fn serialize_using_format(
        &self,
        content_format: &idp_proto::ContentFormat,
        writer: &mut dyn std::io::Write,
    ) -> Result<()> {
        idp_proto::serialize_using_serde_format(self, content_format, writer)
    }
}

impl idp_proto::PlumRelational for OwnedData {
    fn accumulate_plum_relations_nonrecursive(
        &self,
        plum_relation_flags_m: &mut HashMap<PlumHeadSeal, PlumRelationFlags>,
    ) {
        match plum_relation_flags_m.get_mut(&self.data) {
            Some(plum_relation_flags) => {
                *plum_relation_flags |= PlumRelationFlags::CONTENT_DEPENDENCY;
            }
            None => {
                plum_relation_flags_m
                    .insert(self.data.clone(), PlumRelationFlags::CONTENT_DEPENDENCY);
            }
        }
    }
}
