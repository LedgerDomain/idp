use crate::BranchNode;
use anyhow::Result;
use idp_proto::{Plum, PlumHeadSeal};

#[derive(Default)]
pub struct BranchNodeBuilder {
    ancestor_o: Option<PlumHeadSeal>,
    height: u64,
    metadata_o: Option<PlumHeadSeal>,
    content_o: Option<PlumHeadSeal>,
    // TODO: Add posi- and nega-diffs later
}

impl BranchNodeBuilder {
    pub fn new() -> Self {
        let retval = Self::default();
        assert_eq!(retval.height, 0);
        retval
    }
    /// Attempts to build a BranchNode, verifying the field values before returning.
    pub fn build(self) -> Result<BranchNode> {
        // Validate attributes
        anyhow::ensure!(self.metadata_o.is_some(), "BranchNodeBuilder::build can't proceed unless with_metadata was used to specify the metadata PlumHeadSeal");

        Ok(BranchNode {
            ancestor_o: self.ancestor_o,
            height: self.height,
            metadata: self.metadata_o.unwrap(),
            content_o: self.content_o,
            posi_diff_o: None,
            nega_diff_o: None,
        })
    }

    /// Specifies the ancestor Plum for this BranchNode.  The ancestor must itself be a BranchNode.
    pub fn with_ancestor(mut self, ancestor_plum: &Plum) -> Result<Self> {
        // Deserialize the PlumBody into BranchNode, so that we can extract its height and determine its PlumHeadSeal.
        // TEMP HACK -- assume rmp_serde for now.
        use idp_proto::ContentTypeable;
        anyhow::ensure!(BranchNode::content_type_matches(
            ancestor_plum.plum_body.plum_body_content_type.as_slice()
        ));
        let ancestor_branch_node: BranchNode =
            rmp_serde::from_read(ancestor_plum.plum_body.plum_body_content.as_slice())?;
        self.ancestor_o = Some(PlumHeadSeal::from(&ancestor_plum.plum_head));
        self.height = ancestor_branch_node.height + 1;
        Ok(self)
    }
    /// Specifies the metadata field directly.
    pub fn with_metadata(mut self, metadata: PlumHeadSeal) -> Self {
        self.metadata_o = Some(metadata);
        self
    }
    /// Specifies the content field directly.
    pub fn with_content(mut self, content: PlumHeadSeal) -> Self {
        self.content_o = Some(content);
        self
    }
}
