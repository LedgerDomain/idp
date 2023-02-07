use crate::{Nonce, PlumBody};
use anyhow::Result;

#[derive(Default)]
pub struct PlumBodyBuilder {
    body_nonce_o: Option<Nonce>,
    body_content_o: Option<Vec<u8>>,
}

impl PlumBodyBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    /// Attempts to build a PlumBody, verifying the field values before returning.
    pub fn build(self) -> Result<PlumBody> {
        // Validate attributes
        // TODO: Is there any use case for leaving out the body_content?  Perhaps this could mean "empty"
        anyhow::ensure!(self.body_content_o.is_some(), "PlumBodyBuilder::build can't proceed unless with_body_content was used to specify the body content");

        Ok(PlumBody {
            body_nonce_o: self.body_nonce_o,
            body_content: self.body_content_o.unwrap(),
        })
    }

    /// Specifies the body_nonce_o field for use in resisting dictionary attacks.  Default is no nonce.
    pub fn with_body_nonce(mut self, body_nonce: Nonce) -> Self {
        self.body_nonce_o = Some(body_nonce);
        self
    }
    /// Specifies the body_content field.  The body_content field must be defined in order to
    /// build a well-defined PlumBody.
    pub fn with_body_content(mut self, body_content: Vec<u8>) -> Self {
        self.body_content_o = Some(body_content);
        self
    }
    /// Derives the body_content field from a content by serializing it.
    pub fn with_body_content_from<B>(mut self, content: &B) -> Result<Self>
    where
        B: serde::Serialize,
    {
        self.body_content_o = Some(rmp_serde::to_vec(content)?);
        Ok(self)
    }
}
