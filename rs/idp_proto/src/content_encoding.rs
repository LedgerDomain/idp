use crate::{ContentEncoding, Hashable};

impl ContentEncoding {
    /// Convenience method.  ContentEncoding::none() (whose string repr is "") represents no encoding (i.e. no
    /// transformations to the serialized content).
    pub fn none() -> Self {
        Self::from("".to_string())
    }
    /// Convenience method for constructing ContentEncoding for codec "deflate".
    pub fn deflate() -> Self {
        Self::from("deflate".to_string())
    }
    /// Convenience method for constructing ContentEncoding for codec "gzip".
    pub fn gzip() -> Self {
        Self::from("gzip".to_string())
    }
    /// Convenience method for constructing ContentEncoding for codec "identity" (which is a no-op; no transformation).
    pub fn identity() -> Self {
        Self::from("identity".to_string())
    }
    /// A ContentEncoding is a comma-separate string of codec names, where whitespace is ignored.
    /// This normalization strips the whitespace off of each codec name.
    pub fn normalize(&mut self) {
        // This could be implemented better, without allocating.
        self.value = self
            .value
            .split(',')
            .map(|codec| codec.trim())
            .collect::<Vec<_>>()
            .join(",");
    }
}

impl Hashable for ContentEncoding {
    fn update_hasher(&self, hasher: &mut sha2::Sha256) {
        self.value.update_hasher(hasher);
    }
}
