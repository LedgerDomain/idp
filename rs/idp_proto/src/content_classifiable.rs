use crate::{ContentClass, ContentFormat, validate_is_serde_format};
use anyhow::Result;

/// This trait allows a type to have an associated ContentClass.
// TODO: Some types that impl this might have an implied ContentFormat as well (e.g. String -> charset=utf-8)
pub trait ContentClassifiable {
    /// This should define the ContentClass for all instances of the type which implements this trait.
    /// Note that the `Self: Sized` bound is so this method doesn't apply to the trait object.
    // NOTE: The reason this is &'static str and not &'static ContentClass is because ContentClass
    // must be constructed from a String, and str::to_string is not a const function.
    fn content_class_str() -> &'static str
    where
        Self: Sized;
    /// Helper method which should return the same thing as content_class_str(), but can be called
    /// on an instance of this type (e.g. via a trait object).
    fn derive_content_class_str(&self) -> &'static str;
    /// Convenience method for returning the ContentClass (by value).
    fn content_class(&self) -> ContentClass {
        ContentClass::from(self.derive_content_class_str().to_string())
    }

    /// If there is a natural default ContentFormat, return it.  Otherwise None.  If the returned
    /// value is Some(content_format), then self.validate_content_format(&content_format) must not
    /// return error.
    fn default_content_format(&self) -> Option<ContentFormat>;
    /// Validate that the given ContentFormat is a valid format for this type, returning error if not.
    fn validate_content_format(&self, content_format: &ContentFormat) -> Result<()>;

    /// If there is a requested ContentFormat, then this should determine if it's valid,
    /// and if so, return a copy of it (it's not allowed to change the request, if specified).
    /// Otherwise, it can determine its own ContentFormat, if there is one.  It could
    /// also return an error if there is no valid ContentFormat.
    fn determine_content_format(
        &self,
        requested_content_format_o: Option<&ContentFormat>,
    ) -> Result<ContentFormat> {
        if let Some(requested_content_format) = requested_content_format_o {
            self.validate_content_format(requested_content_format)?;
            Ok(requested_content_format.clone())
        } else {
            self.default_content_format().ok_or_else(|| anyhow::anyhow!("No ContentFormat was requested and there is no default ContentFormat for ContentClass {:?}", self.derive_content_class_str()))
        }
    }
}

/// Impl of ContentClassifiable for common type String.
// TODO: This doesn't handle specifying "charset=utf-8" as the format.
impl ContentClassifiable for String {
    fn content_class_str() -> &'static str {
        <&str>::content_class_str()
    }
    fn derive_content_class_str(&self) -> &'static str {
        self.as_str().derive_content_class_str()
    }
    fn default_content_format(&self) -> Option<ContentFormat> {
        self.as_str().default_content_format()
    }
    fn validate_content_format(&self, content_format: &ContentFormat) -> Result<()> {
        self.as_str().validate_content_format(content_format)
    }
}

/// Impl of ContentClassifiable for common type &str.
// TODO: This doesn't handle specifying "charset=utf-8" as the format.
impl ContentClassifiable for &str {
    fn content_class_str() -> &'static str {
        "text/plain"
    }
    fn derive_content_class_str(&self) -> &'static str {
        "text/plain"
    }
    fn default_content_format(&self) -> Option<ContentFormat> {
        if self.is_ascii() {
            Some(ContentFormat::charset_us_ascii())
        } else {
            Some(ContentFormat::charset_utf_8())
        }
    }
    fn validate_content_format(&self, content_format: &ContentFormat) -> Result<()> {
        match content_format.as_str() {
            "charset=us-ascii" => {
                anyhow::ensure!(
                    self.is_ascii(),
                    "ContentFormat {:?} is not valid for a string containing non-ascii chars",
                    "charset=us-ascii"
                );
                Ok(())
            }
            "charset=utf-8" => Ok(()),
            _ => validate_is_serde_format(content_format),
        }
    }
}

/// Impl of ContentClassifiable for common type Vec<u8>.
impl ContentClassifiable for Vec<u8> {
    fn content_class_str() -> &'static str {
        <&[u8]>::content_class_str()
    }
    fn derive_content_class_str(&self) -> &'static str {
        self.as_slice().derive_content_class_str()
    }
    fn default_content_format(&self) -> Option<ContentFormat> {
        self.as_slice().default_content_format()
    }
    fn validate_content_format(&self, content_format: &ContentFormat) -> Result<()> {
        self.as_slice().validate_content_format(content_format)
    }
}

/// Impl of ContentClassifiable for common type &[u8].
impl ContentClassifiable for &[u8] {
    fn content_class_str() -> &'static str {
        "application/octet-stream"
    }
    fn derive_content_class_str(&self) -> &'static str {
        "application/octet-stream"
    }
    fn default_content_format(&self) -> Option<ContentFormat> {
        Some(ContentFormat::none())
    }
    fn validate_content_format(&self, content_format: &ContentFormat) -> Result<()> {
        match content_format.as_str() {
            "" => Ok(()),
            _ => validate_is_serde_format(content_format),
        }
    }
}
