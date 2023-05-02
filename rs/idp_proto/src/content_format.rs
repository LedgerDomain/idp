use crate::{ContentFormat, Hashable};

impl ContentFormat {
    /// Convenience method.  ContentEncoding::none() (whose string repr is "") represents no format (e.g.
    /// for unstructured bytes, or otherwise unspecified format).
    pub fn none() -> Self {
        Self::from("".to_string())
    }
    pub fn charset_us_ascii() -> Self {
        Self::from("charset=us-ascii".to_string())
    }
    pub fn charset_utf_8() -> Self {
        Self::from("charset=utf-8".to_string())
    }
    pub fn json() -> Self {
        Self::from("json".to_string())
    }
    pub fn msgpack() -> Self {
        Self::from("msgpack".to_string())
    }
}

impl Hashable for ContentFormat {
    fn update_hasher(&self, hasher: &mut sha2::Sha256) {
        self.value.update_hasher(hasher);
    }
}
