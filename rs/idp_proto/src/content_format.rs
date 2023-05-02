use crate::{ContentFormat, Hashable};
use anyhow::Result;

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

/// Returns error if the given ContentFormat does not correspond to a known serde format (e.g. "json", "msgpack").
/// Note that this doesn't take into account which formats are enabled using feature flags.
pub fn validate_is_serde_format(content_format: &ContentFormat) -> Result<()> {
    match content_format.as_str() {
        "json" | "msgpack" => Ok(()),
        _ => {
            anyhow::bail!(
                "ContentFormat {:?} doesn't correspond to a known (to idp_proto) serde format",
                content_format.as_str()
            );
        }
    }
}
