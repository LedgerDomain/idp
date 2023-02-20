use crate::{
    ContentType, ContentTypeable, Id, Nonce, Plum, PlumBodyBuilder, PlumBodySeal, PlumHeadBuilder,
    PlumRelational, PlumRelationsBuilder, PlumRelationsSeal, UnixNanoseconds,
};
use anyhow::Result;

#[derive(Default)]
pub struct PlumBuilder {
    plum_head_builder: PlumHeadBuilder,
    plum_relations_builder: PlumRelationsBuilder,
    plum_body_builder: PlumBodyBuilder,
}

impl PlumBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    /// Attempts to build a Plum, verifying the field values before returning.
    pub fn build(mut self) -> Result<Plum> {
        let plum_body = self.plum_body_builder.build()?;
        let plum_body_seal = PlumBodySeal::from(&plum_body);

        self.plum_relations_builder = self
            .plum_relations_builder
            .with_source_plum_body_seal(plum_body_seal.clone());
        let plum_relations_o = self.plum_relations_builder.build()?;

        let plum_head_builder = match &plum_relations_o {
            Some(plum_relations) => self
                .plum_head_builder
                .with_plum_relations_seal(PlumRelationsSeal::from(plum_relations)),
            None => self.plum_head_builder,
        };
        let plum_head = plum_head_builder
            .with_plum_body_seal(plum_body_seal)
            .build()?;

        Ok(Plum {
            plum_head,
            plum_relations_o,
            plum_body,
        })
    }

    /// Convenience method which derives the head.plum_body_content_type, plum_relations_o, and body.plum_body_content
    /// fields from content whose type implements the Relational, ContentTypeable, and serde::Serialize traits.
    /// See PlumHeadBuilder::with_plum_body_content_type_from, PlumRelationsBuilder::with_plum_relations_from,
    /// and PlumBodyBuilder::with_plum_body_content_from.
    pub fn with_relational_typed_content_from<B>(mut self, content: &B) -> Result<Self>
    where
        B: ContentTypeable + PlumRelational + serde::Serialize,
    {
        self.plum_relations_builder = self
            .plum_relations_builder
            .with_plum_relations_from(content);
        self.plum_body_builder = self
            .plum_body_builder
            .with_plum_body_content_type_from(content)
            .with_plum_body_content_from(content)?;
        Ok(self)
    }
    /// Convenience method which derives the head.plum_body_content_type and body.plum_body_content fields from
    /// content whose type implements the ContentTypeable and serde::Serialize traits, but not necessarily
    /// the Relational trait.  Note that you only want this if this Plum is meant to have no plum_relations.
    /// See PlumHeadBuilder::with_plum_body_content_type_from and PlumBodyBuilder::with_plum_body_content_from.
    pub fn with_nonrelational_typed_content_from<B>(mut self, content: &B) -> Result<Self>
    where
        B: ContentTypeable + serde::Serialize,
    {
        self.plum_body_builder = self
            .plum_body_builder
            .with_plum_body_content_type_from(content)
            .with_plum_body_content_from(content)?;
        Ok(self)
    }
    /// Convenience method which derives the plum_relations_o and and body.plum_body_content fields from content
    /// whose type implements the Relational and serde::Serialize traits, but not necessarily the
    /// ContentTypeable trait.  Note that you only want this if this Plum is meant to have no
    /// plum_body_content_type (default plum_body_content_type is "application/octet-stream").
    /// See PlumRelationsBuilder::with_plum_relations_from and PlumBodyBuilder::with_plum_body_content_from.
    pub fn with_relational_untyped_content_from<B>(mut self, content: &B) -> Result<Self>
    where
        B: PlumRelational + serde::Serialize,
    {
        self.plum_relations_builder = self
            .plum_relations_builder
            .with_plum_relations_from(content);
        self.plum_body_builder = self
            .plum_body_builder
            .with_plum_body_content_from(content)?;
        Ok(self)
    }
    /// Convenience method which derives the body.plum_body_content field from content whose type implements
    /// the serde::Serialize trait, but not necessarily the ContentTypeable or Relational trait.  Note
    /// that you only want this if this Plum is meant to have no plum_relations and no plum_body_content_type
    /// (default plum_body_content_type is "application/octet-stream").
    /// See PlumBodyBuilder::with_plum_body_content_from.
    pub fn with_nonrelational_untyped_content_from<B>(mut self, content: &B) -> Result<Self>
    where
        B: ContentTypeable + serde::Serialize,
    {
        self.plum_body_builder = self
            .plum_body_builder
            .with_plum_body_content_from(content)?;
        Ok(self)
    }

    /// Specifies the head.plum_body_content_type field directly.
    pub fn with_plum_body_content_type(mut self, plum_body_content_type: ContentType) -> Self {
        self.plum_body_builder = self
            .plum_body_builder
            .with_plum_body_content_type(plum_body_content_type);
        self
    }
    /// Specifies the head.head_nonce_o field directly.  Default is None.
    pub fn with_plum_head_nonce(mut self, head_nonce: Nonce) -> Self {
        self.plum_head_builder = self.plum_head_builder.with_plum_head_nonce(head_nonce);
        self
    }
    /// Specifies the head.owner_id_o field directly.  Default is None.
    pub fn with_owner_id(mut self, owner_id: Id) -> Self {
        self.plum_head_builder = self.plum_head_builder.with_owner_id(owner_id);
        self
    }
    /// Specifies the head.created_at_o field directly.  Default is None.
    pub fn with_created_at(mut self, created_at: UnixNanoseconds) -> Self {
        self.plum_head_builder = self.plum_head_builder.with_created_at(created_at);
        self
    }
    /// Specifies the head.metadata_o field directly.  Default is None.
    pub fn with_metadata(mut self, metadata: Vec<u8>) -> Self {
        self.plum_head_builder = self.plum_head_builder.with_metadata(metadata);
        self
    }
    /// Specifies the plum_relations_o.plum_relations_nonce_o field for use in resisting dictionary attacks.
    /// Default is no nonce.
    pub fn with_plum_relations_nonce(mut self, plum_relations_nonce: Nonce) -> Self {
        self.plum_relations_builder = self
            .plum_relations_builder
            .with_plum_relations_nonce(plum_relations_nonce);
        self
    }
    /// Specifies the body.plum_body_nonce_o field directly.  Default is None.
    pub fn with_plum_body_nonce(mut self, plum_body_nonce: Nonce) -> Self {
        self.plum_body_builder = self.plum_body_builder.with_plum_body_nonce(plum_body_nonce);
        self
    }
    /// Specifies the body.plum_body_content field directly.
    pub fn with_plum_body_content(mut self, plum_body_content: Vec<u8>) -> Self {
        self.plum_body_builder = self
            .plum_body_builder
            .with_plum_body_content(plum_body_content);
        self
    }
}
