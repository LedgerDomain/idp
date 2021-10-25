use crate::{
    ContentType, ContentTypeable, Did, Nonce, PlumBody, PlumBodySeal, PlumHead,
    PlumRelations, PlumRelationsSeal, UnixSeconds,
};

#[derive(Default)]
pub struct PlumHeadBuilder {
    body_seal_o: Option<PlumBodySeal>,
    body_content_type_o: Option<ContentType>,
    body_length_o: Option<u64>,
    head_nonce_o: Option<Nonce>,
    owner_did_o: Option<Did>,
    created_at_o: Option<UnixSeconds>,
    metadata_o: Option<Vec<u8>>,
    relations_seal_o: Option<PlumRelationsSeal>,
}

impl PlumHeadBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    /// Attempts to build a PlumHead, verifying the field values before returning.
    pub fn build(self) -> Result<PlumHead, failure::Error> {
        // Validate attributes
        failure::ensure!(self.body_seal_o.is_some(), "PlumHeadBuilder::build can't proceed unless with_body_seal was used to specify the PlumBodySeal");
        failure::ensure!(self.body_length_o.is_some(), "PlumHeadBuilder::build can't proceed unless with_body_length was used to specify the body length");

        Ok(PlumHead {
            body_seal: self.body_seal_o.unwrap(),
            // Default is byte stream, which is as unstructured as it gets.
            body_content_type: self.body_content_type_o.unwrap_or(ContentType::from("application/octet-stream")),
            body_length: self.body_length_o.unwrap(),
            head_nonce_o: self.head_nonce_o,
            owner_did_o: self.owner_did_o,
            created_at_o: self.created_at_o,
            metadata_o: self.metadata_o,
            relations_seal_o: self.relations_seal_o,
        })
    }

    /// Derives the body_seal and body_length fields from a PlumBody.  This is a convenience
    /// builder method for when you have the full PlumBody available.
    pub fn with_body(self, body: &PlumBody) -> Self {
        self.with_body_seal(PlumBodySeal::from(body))
            .with_body_length(body.body_content.len() as u64)
    }
    /// Specifies the PlumBodySeal directly.  The body_seal field must be defined in order
    /// to build a well-defined PlumHead.
    pub fn with_body_seal(mut self, body_seal: PlumBodySeal) -> Self {
        self.body_seal_o = Some(body_seal);
        self
    }
    /// Specifies the body_content_type field directly.
    pub fn with_body_content_type(mut self, body_content_type: ContentType) -> Self {
        self.body_content_type_o = Some(body_content_type);
        self
    }
    /// Derives the body_content_type field from a typed body that has the ContentTypeable trait.
    pub fn with_body_content_type_from<B>(self, content: &B) -> Self
    where B: ContentTypeable {
        self.with_body_content_type(content.derive_content_type())
    }
    /// Specifies the body_length field directly.  The body_length field must be defined in order
    /// to build a well-defined PlumHead.
    pub fn with_body_length(mut self, body_length: u64) -> Self {
        self.body_length_o = Some(body_length);
        self
    }
    /// Specifies the head_nonce_o field directly.
    pub fn with_head_nonce(mut self, head_nonce: Nonce) -> Self {
        self.head_nonce_o = Some(head_nonce);
        self
    }
    /// Specifies the owner_did_o field directly.
    pub fn with_owner_did(mut self, owner_did: Did) -> Self {
        self.owner_did_o = Some(owner_did);
        self
    }
    /// Specifies the created_at_o field directly.
    pub fn with_created_at(mut self, created_at: UnixSeconds) -> Self {
        self.created_at_o = Some(created_at);
        self
    }
    /// Specifies the metadata_o field directly.
    pub fn with_metadata(mut self, metadata: Vec<u8>) -> Self {
        self.metadata_o = Some(metadata);
        self
    }
    /// Specifies the relations_seal_o field directly.
    pub fn with_relations_seal(mut self, relations_seal: PlumRelationsSeal) -> Self {
        self.relations_seal_o = Some(relations_seal);
        self
    }
    /// Derives the relations_seal_o field from a PlumRelations.  This is a convenience builder
    /// method for when you have the full PlumRelations available.
    pub fn with_relations(self, relations: &PlumRelations) -> Self {
        self.with_relations_seal(PlumRelationsSeal::from(relations))
    }
}

