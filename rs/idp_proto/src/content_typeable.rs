use crate::ContentType;

/// This trait defines how to derive ContentType for a given type.
// TODO: Consider making this `ContentTypeable: 'static`
pub trait ContentTypeable {
    /// This should define the ContentType for all instances of the type which implements this trait.
    fn content_type() -> ContentType;
    /// Helper method which simply calls content_type() on a particular instance of the type.
    fn derive_content_type(&self) -> ContentType {
        Self::content_type()
    }
    /// Helper method for checking if a given &[u8] matches the ContentType of Self.
    fn content_type_matches(bytes: &[u8]) -> bool;
}
