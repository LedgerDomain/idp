use crate::ContentType;

/// This trait defines how to derive ContentType for a given type.
pub trait ContentTypeable {
    /// This should define the ContentType for all instances of the type which implements this trait.
    fn content_type() -> ContentType;
    /// Helper method which simply calls content_type() on a particular instance of the type.
    fn derive_content_type(&self) -> ContentType {
        Self::content_type()
    }
}
