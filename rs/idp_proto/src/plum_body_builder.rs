use crate::{ContentType, ContentTypeable, Nonce, PlumBody};
use anyhow::Result;

#[derive(Default)]
pub struct PlumBodyBuilder {
    plum_body_nonce_o: Option<Nonce>,
    plum_body_content_length_o: Option<u64>,
    plum_body_content_type_o: Option<ContentType>,
    plum_body_content_o: Option<Vec<u8>>,
}

impl PlumBodyBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    /// Attempts to build a PlumBody, verifying the field values before returning.
    pub fn build(self) -> Result<PlumBody> {
        // Validate attributes
        anyhow::ensure!(self.plum_body_content_length_o.is_some(), "PlumBodyBuilder::build can't proceed unless with_plum_body_content_length was used to specify the body length");
        // anyhow::ensure!(self.plum_body_content_type_o.is_some(), "PlumBodyBuilder::build can't proceed unless with_plum_body_content_type was used to specify the body content type");
        // TODO: Is there any use case for leaving out the plum_body_content?  Perhaps this could mean "empty"
        anyhow::ensure!(self.plum_body_content_o.is_some(), "PlumBodyBuilder::build can't proceed unless with_plum_body_content was used to specify the body content");

        Ok(PlumBody {
            plum_body_nonce_o: self.plum_body_nonce_o,
            plum_body_content_length: self.plum_body_content_length_o.unwrap(),
            // Default is byte stream, which is as unstructured as it gets.
            plum_body_content_type: self.plum_body_content_type_o.unwrap_or(ContentType::from(
                "application/octet-stream".as_bytes().to_vec(),
            )),
            plum_body_content: self.plum_body_content_o.unwrap(),
        })
    }

    /// Specifies the plum_body_nonce_o field for use in resisting dictionary attacks.  Default is no nonce.
    pub fn with_plum_body_nonce(mut self, plum_body_nonce: Nonce) -> Self {
        self.plum_body_nonce_o = Some(plum_body_nonce);
        self
    }
    /// Specifies the plum_body_content_type field directly.
    pub fn with_plum_body_content_type(mut self, plum_body_content_type: ContentType) -> Self {
        self.plum_body_content_type_o = Some(plum_body_content_type);
        self
    }
    /// Derives the plum_body_content_type field from a typed body that has the ContentTypeable trait.
    pub fn with_plum_body_content_type_from<B>(self, content: &B) -> Self
    where
        B: ContentTypeable,
    {
        self.with_plum_body_content_type(content.derive_content_type())
    }
    /// Specifies the plum_body_length field directly.  The plum_body_length field must be defined in order
    /// to build a well-defined PlumHead.
    pub fn with_plum_body_content_length(mut self, plum_body_length: u64) -> Self {
        self.plum_body_content_length_o = Some(plum_body_length);
        self
    }
    /// Specifies the plum_body_content field.  The plum_body_content field must be defined in order to
    /// build a well-defined PlumBody.
    pub fn with_plum_body_content(mut self, plum_body_content: Vec<u8>) -> Self {
        self.plum_body_content_length_o = Some(plum_body_content.len() as u64);
        self.plum_body_content_o = Some(plum_body_content);
        self
    }
    /// Derives the plum_body_content field from a content by serializing it.
    pub fn with_plum_body_content_from<B>(mut self, content: &B) -> Result<Self>
    where
        B: serde::Serialize,
    {
        let plum_body_content = rmp_serde::to_vec(content)?;
        self.plum_body_content_length_o = Some(plum_body_content.len() as u64);
        self.plum_body_content_o = Some(plum_body_content);
        Ok(self)
    }
}
