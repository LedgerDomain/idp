use crate::{ContentType, Did, Nonce, Plum, PlumBody, PlumBodySeal, PlumHead, UnixSeconds};

#[derive(Default)]
pub struct PlumHeadBuilder {
    body_seal_o: Option<PlumBodySeal>,
    body_content_type_o: Option<ContentType>,
    body_length_o: Option<u64>,
    head_nonce_o: Option<Nonce>,
    owner_did_o: Option<Did>,
    created_at_o: Option<UnixSeconds>,
    metadata_o: Option<Vec<u8>>,
}

impl PlumHeadBuilder {
    pub fn new() -> Self {
        Self::default()
    }
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
        })
    }
    pub fn build_with_body(self, body: &PlumBody) -> Result<PlumHead, failure::Error> {
        Ok(
            self.with_body_seal(PlumBodySeal::from(body))
                .with_body_length(body.body_content.len() as u64)
                .build()?
        )
    }

    pub fn with_body_seal(mut self, body_seal: PlumBodySeal) -> Self {
        self.body_seal_o = Some(body_seal);
        self
    }
    pub fn with_body_content_type(mut self, body_content_type: ContentType) -> Self {
        self.body_content_type_o = Some(body_content_type);
        self
    }
    pub fn with_body_length(mut self, body_length: u64) -> Self {
        self.body_length_o = Some(body_length);
        self
    }
    pub fn with_head_nonce(mut self, head_nonce: Nonce) -> Self {
        self.head_nonce_o = Some(head_nonce);
        self
    }
    pub fn with_owner_did(mut self, owner_did: Did) -> Self {
        self.owner_did_o = Some(owner_did);
        self
    }
    pub fn with_created_at(mut self, created_at: UnixSeconds) -> Self {
        self.created_at_o = Some(created_at);
        self
    }
    pub fn with_metadata(mut self, metadata: Vec<u8>) -> Self {
        self.metadata_o = Some(metadata);
        self
    }
}

#[derive(Default)]
pub struct PlumBodyBuilder {
    body_nonce_o: Option<Nonce>,
    body_content_o: Option<Vec<u8>>,
}

impl PlumBodyBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn build(self) -> Result<PlumBody, failure::Error> {
        // Validate attributes
        failure::ensure!(self.body_content_o.is_some(), "PlumBodyBuilder::build can't proceed unless with_body_content was used to specify the body content");

        Ok(PlumBody {
            body_nonce_o: self.body_nonce_o,
            body_content: self.body_content_o.unwrap(),
        })
    }

    pub fn with_body_nonce(mut self, body_nonce: Nonce) -> Self {
        self.body_nonce_o = Some(body_nonce);
        self
    }
    pub fn with_body_content(mut self, body_content: Vec<u8>) -> Self {
        self.body_content_o = Some(body_content);
        self
    }
}

#[derive(Default)]
pub struct PlumBuilder {
    head_builder: PlumHeadBuilder,
    body_builder: PlumBodyBuilder,
}

impl PlumBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn build(self) -> Result<Plum, failure::Error> {
        let body = self.body_builder.build()?;
        let head = self.head_builder.build_with_body(&body)?;
        Ok(Plum { head, body })
    }

    pub fn with_body_content_type(mut self, body_content_type: ContentType) -> Self {
        self.head_builder = self.head_builder.with_body_content_type(body_content_type);
        self
    }
    pub fn with_head_nonce(mut self, head_nonce: Nonce) -> Self {
        self.head_builder = self.head_builder.with_head_nonce(head_nonce);
        self
    }
    pub fn with_owner_did(mut self, owner_did: Did) -> Self {
        self.head_builder = self.head_builder.with_owner_did(owner_did);
        self
    }
    pub fn with_created_at(mut self, created_at: UnixSeconds) -> Self {
        self.head_builder = self.head_builder.with_created_at(created_at);
        self
    }
    pub fn with_metadata(mut self, metadata: Vec<u8>) -> Self {
        self.head_builder = self.head_builder.with_metadata(metadata);
        self
    }
    pub fn with_body_nonce(mut self, body_nonce: Nonce) -> Self {
        self.body_builder = self.body_builder.with_body_nonce(body_nonce);
        self
    }
    pub fn with_body_content(mut self, body_content: Vec<u8>) -> Self {
        self.body_builder = self.body_builder.with_body_content(body_content);
        self
    }
}
