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
}

/// Impl of ContentClassifiable for common type String.
// TODO: This doesn't handle specifying "charset=utf-8" as the format.
impl ContentClassifiable for String {
    fn content_class_str() -> &'static str {
        "text/plain"
    }
    fn derive_content_class_str(&self) -> &'static str {
        Self::content_class_str()
    }
}

/// Impl of ContentClassifiable for common type &str.
// TODO: This doesn't handle specifying "charset=utf-8" as the format.
impl ContentClassifiable for &str {
    fn content_class_str() -> &'static str {
        "text/plain"
    }
    fn derive_content_class_str(&self) -> &'static str {
        Self::content_class_str()
    }
}

/// Impl of ContentClassifiable for common type Vec<u8>.
impl ContentClassifiable for Vec<u8> {
    fn content_class_str() -> &'static str {
        "application/octet-stream"
    }
    fn derive_content_class_str(&self) -> &'static str {
        Self::content_class_str()
    }
}

/// Impl of ContentClassifiable for common type &[u8].
impl ContentClassifiable for &[u8] {
    fn content_class_str() -> &'static str {
        "application/octet-stream"
    }
    fn derive_content_class_str(&self) -> &'static str {
        Self::content_class_str()
    }
}
