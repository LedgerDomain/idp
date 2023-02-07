use crate::{
    ContentType, ContentTypeable, Did, Nonce, Plum, PlumBodyBuilder, PlumHeadBuilder,
    PlumRelationsBuilder, PlumRelationsSeal, Relational, UnixSeconds,
};
use anyhow::Result;

#[derive(Default)]
pub struct PlumBuilder {
    head_builder: PlumHeadBuilder,
    relations_builder: PlumRelationsBuilder,
    body_builder: PlumBodyBuilder,
}

impl PlumBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    /// Attempts to build a Plum, verifying the field values before returning.
    pub fn build(self) -> Result<Plum> {
        let body = self.body_builder.build()?;
        let relations_o = self.relations_builder.build();
        let head_builder = match &relations_o {
            Some(relations) => self
                .head_builder
                .with_relations_seal(PlumRelationsSeal::from(relations)),
            None => self.head_builder,
        };
        let head = head_builder.with_body(&body).build()?;
        Ok(Plum {
            head,
            relations_o,
            body,
        })
    }

    /// Convenience method which derives the head.body_content_type, relations_o, and body.body_content
    /// fields from content whose type implements the Relational, ContentTypeable, and serde::Serialize traits.
    /// See PlumHeadBuilder::with_body_content_type_from, PlumRelationsBuilder::with_relations_from,
    /// and PlumBodyBuilder::with_body_content_from.
    pub fn with_relational_typed_content_from<B>(mut self, content: &B) -> Result<Self>
    where
        B: ContentTypeable + Relational + serde::Serialize,
    {
        self.head_builder = self.head_builder.with_body_content_type_from(content);
        self.relations_builder = self.relations_builder.with_relations_from(content);
        self.body_builder = self.body_builder.with_body_content_from(content)?;
        Ok(self)
    }
    /// Convenience method which derives the head.body_content_type and body.body_content fields from
    /// content whose type implements the ContentTypeable and serde::Serialize traits, but not necessarily
    /// the Relational trait.  Note that you only want this if this Plum is meant to have no relations.
    /// See PlumHeadBuilder::with_body_content_type_from and PlumBodyBuilder::with_body_content_from.
    pub fn with_nonrelational_typed_content_from<B>(mut self, content: &B) -> Result<Self>
    where
        B: ContentTypeable + serde::Serialize,
    {
        self.head_builder = self.head_builder.with_body_content_type_from(content);
        self.body_builder = self.body_builder.with_body_content_from(content)?;
        Ok(self)
    }
    /// Convenience method which derives the relations_o and and body.body_content fields from content
    /// whose type implements the Relational and serde::Serialize traits, but not necessarily the
    /// ContentTypeable trait.  Note that you only want this if this Plum is meant to have no
    /// body_content_type (default body_content_type is "application/octet-stream").
    /// See PlumRelationsBuilder::with_relations_from and PlumBodyBuilder::with_body_content_from.
    pub fn with_relational_untyped_content_from<B>(mut self, content: &B) -> Result<Self>
    where
        B: Relational + serde::Serialize,
    {
        self.relations_builder = self.relations_builder.with_relations_from(content);
        self.body_builder = self.body_builder.with_body_content_from(content)?;
        Ok(self)
    }
    /// Convenience method which derives the body.body_content field from content whose type implements
    /// the serde::Serialize trait, but not necessarily the ContentTypeable or Relational trait.  Note
    /// that you only want this if this Plum is meant to have no relations and no body_content_type
    /// (default body_content_type is "application/octet-stream").
    /// See PlumBodyBuilder::with_body_content_from.
    pub fn with_nonrelational_untyped_content_from<B>(mut self, content: &B) -> Result<Self>
    where
        B: ContentTypeable + serde::Serialize,
    {
        self.body_builder = self.body_builder.with_body_content_from(content)?;
        Ok(self)
    }

    /// Specifies the head.body_content_type field directly.
    pub fn with_body_content_type(mut self, body_content_type: ContentType) -> Self {
        self.head_builder = self.head_builder.with_body_content_type(body_content_type);
        self
    }
    /// Specifies the head.head_nonce_o field directly.  Default is None.
    pub fn with_head_nonce(mut self, head_nonce: Nonce) -> Self {
        self.head_builder = self.head_builder.with_head_nonce(head_nonce);
        self
    }
    /// Specifies the head.owner_did_o field directly.  Default is None.
    pub fn with_owner_did(mut self, owner_did: Did) -> Self {
        self.head_builder = self.head_builder.with_owner_did(owner_did);
        self
    }
    /// Specifies the head.created_at_o field directly.  Default is None.
    pub fn with_created_at(mut self, created_at: UnixSeconds) -> Self {
        self.head_builder = self.head_builder.with_created_at(created_at);
        self
    }
    /// Specifies the head.metadata_o field directly.  Default is None.
    pub fn with_metadata(mut self, metadata: Vec<u8>) -> Self {
        self.head_builder = self.head_builder.with_metadata(metadata);
        self
    }
    /// Specifies the relations_o.relations_nonce_o field for use in resisting dictionary attacks.
    /// Default is no nonce.
    pub fn with_relations_nonce(mut self, relations_nonce: Nonce) -> Self {
        self.relations_builder = self.relations_builder.with_relations_nonce(relations_nonce);
        self
    }
    /// Specifies the body.body_nonce_o field directly.  Default is None.
    pub fn with_body_nonce(mut self, body_nonce: Nonce) -> Self {
        self.body_builder = self.body_builder.with_body_nonce(body_nonce);
        self
    }
    /// Specifies the body.body_content field directly.
    pub fn with_body_content(mut self, body_content: Vec<u8>) -> Self {
        self.body_builder = self.body_builder.with_body_content(body_content);
        self
    }
}
