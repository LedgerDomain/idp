use crate::{
    serialize_and_encode_to_content, Content, ContentEncoding, ContentFormat, Nonce, Plum,
    PlumBody, PlumBodySeal, PlumHead, PlumMetadata, PlumMetadataSeal, PlumRelational,
    PlumRelationsBuilder, PlumRelationsSeal, Serializable, UnixNanoseconds,
};
use anyhow::Result;

pub struct PlumBuilder {
    plum_head_nonce_o: Option<Nonce>,
    plum_metadata_nonce_o: Option<Nonce>,
    plum_created_at_o: Option<UnixNanoseconds>,
    plum_metadata_should_include_plum_body_content_metadata: bool,
    plum_metadata_additional_content_o: Option<Content>,
    plum_relations_builder: PlumRelationsBuilder,
    plum_body_nonce_o: Option<Nonce>,
    plum_body_content_o: Option<Content>,
}

impl Default for PlumBuilder {
    fn default() -> Self {
        Self {
            plum_head_nonce_o: None,
            plum_metadata_nonce_o: None,
            plum_created_at_o: None,
            plum_metadata_should_include_plum_body_content_metadata: true,
            plum_metadata_additional_content_o: None,
            plum_relations_builder: PlumRelationsBuilder::new(),
            plum_body_nonce_o: None,
            plum_body_content_o: None,
        }
    }
}

impl PlumBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    /// Attempts to build a Plum, verifying the field values before returning.  Note that this automatically
    /// sets the plum_body_content_length_o and plum_body_content_type_o fields of the PlumMetadataBuilder
    /// before building.
    // TODO: Figure out how to specify that it should leave these fields as None.
    pub fn build(mut self) -> Result<Plum> {
        // Validate attributes.
        anyhow::ensure!(self.plum_body_content_o.is_some(), "PlumBuilder::build can't proceed unless the with_plum_body_content_from or with_plum_body_content method was used to specify the plum body content.");

        let plum_body = PlumBody {
            plum_body_nonce_o: self.plum_body_nonce_o,
            plum_body_content: self.plum_body_content_o.unwrap(),
        };
        let plum_body_seal = PlumBodySeal::from(&plum_body);

        self.plum_relations_builder = self
            .plum_relations_builder
            .with_source_plum_body_seal(plum_body_seal.clone());
        let plum_relations = self.plum_relations_builder.build()?;
        let plum_relations_seal = PlumRelationsSeal::from(&plum_relations);

        let plum_body_content_metadata_o =
            if self.plum_metadata_should_include_plum_body_content_metadata {
                Some(plum_body.plum_body_content.content_metadata.clone())
            } else {
                None
            };

        let plum_metadata = PlumMetadata {
            plum_metadata_nonce_o: self.plum_metadata_nonce_o,
            plum_created_at_o: self.plum_created_at_o,
            plum_body_content_metadata_o,
            additional_content_o: self.plum_metadata_additional_content_o,
        };
        let plum_metadata_seal = PlumMetadataSeal::from(&plum_metadata);

        let plum_head = PlumHead {
            plum_head_nonce_o: self.plum_head_nonce_o,
            plum_metadata_seal,
            plum_relations_seal,
            plum_body_seal,
        };

        Ok(Plum {
            plum_head,
            plum_metadata,
            plum_relations,
            plum_body,
        })
    }

    /// Specifies the plum_head_nonce_o field directly.  Default is None.
    pub fn with_plum_head_nonce(mut self, plum_head_nonce: Nonce) -> Self {
        self.plum_head_nonce_o = Some(plum_head_nonce);
        self
    }
    /// Specifies the plum_metadata_nonce_o field directly.  Default is None.
    pub fn with_plum_metadata_nonce(mut self, plum_metadata_nonce: Nonce) -> Self {
        self.plum_metadata_nonce_o = Some(plum_metadata_nonce);
        self
    }
    /// Specifies the plum_created_at field (of PlumMetadata) directly.  Default is None.
    pub fn with_plum_created_at(mut self, plum_created_at: UnixNanoseconds) -> Self {
        self.plum_created_at_o = Some(plum_created_at);
        self
    }
    /// Specifies that the plum_body_content_metadata field (of PlumMetadata) should NOT be set
    /// to the a copy of plum_body_content.content_metadata field.  The default is to set it,
    /// since that is convenient and probably most of the time you want it that way.  The reason
    /// why you might not want to include it is if you don't want to leak any information about
    /// the PlumBody in the (mandatory in the IDP data model) PlumMetadata, for example, if you
    /// were in a situation where you had to make public PlumHead records of PlumBody-s that
    /// are secret but may need to be audited by a third party at some point.
    pub fn plum_metadata_should_not_include_plum_body_content_metadata(mut self) -> Self {
        self.plum_metadata_should_include_plum_body_content_metadata = false;
        self
    }
    /// Specifies the additional_content field (of PlumMetadata) directly.  Default is None.
    pub fn with_plum_metadata_additional_content(mut self, additional_content: Content) -> Self {
        self.plum_metadata_additional_content_o = Some(additional_content);
        self
    }
    /// Specifies the plum_relations_nonce_o field in for use in resisting dictionary attacks.
    /// Default is no nonce.
    pub fn with_plum_relations_nonce(mut self, plum_relations_nonce: Nonce) -> Self {
        self.plum_relations_builder = self
            .plum_relations_builder
            .with_plum_relations_nonce(plum_relations_nonce);
        self
    }
    /// Specifies the plum_body_nonce_o field directly.  Default is None.
    pub fn with_plum_body_nonce(mut self, plum_body_nonce: Nonce) -> Self {
        self.plum_body_nonce_o = Some(plum_body_nonce);
        self
    }
    /// Convenience method which derives the plum relations and the plum body content from the given
    /// value.  This does not alter the plum relations nonce or the plum body nonce.
    pub fn with_plum_relations_and_plum_body_content_from<'a, T>(
        self,
        value: &T,
        content_format: ContentFormat,
        content_encoding: ContentEncoding,
    ) -> Result<Self>
    where
        T: PlumRelational + Serializable,
    {
        self.with_plum_relations_from(value)?
            .with_plum_body_content_from(value, content_format, content_encoding)
    }
    /// Convenience method which derives the plum relations from the given value.  This does not alter
    /// the plum relations nonce.
    fn with_plum_relations_from<'a, T>(mut self, value: &T) -> Result<Self>
    where
        T: PlumRelational,
    {
        self.plum_relations_builder = self.plum_relations_builder.with_plum_relations_from(value);
        Ok(self)
    }
    /// Specifies the plum_body_content field directly.  You probably want to use with_plum_body_content_from,
    /// as it's more convenient.
    fn with_plum_body_content(mut self, plum_body_content: Content) -> Self {
        self.plum_body_content_o = Some(plum_body_content);
        self
    }
    /// Convenience method which derives the plum body content from the given value, format, and encoding(s).
    /// This does not alter the plum body nonce.
    fn with_plum_body_content_from<T>(
        self,
        value: &T,
        content_format: ContentFormat,
        mut content_encoding: ContentEncoding,
    ) -> Result<Self>
    where
        T: Serializable,
    {
        content_encoding.normalize();
        Ok(self.with_plum_body_content(serialize_and_encode_to_content(
            value,
            content_format,
            content_encoding,
        )?))
    }
}
