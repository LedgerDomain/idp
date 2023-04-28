#[derive(Clone, Copy, Debug, thiserror::Error)]
pub enum LoadPlumAndDeserializeError {
    #[error("ContentClass mismatch")]
    ContentClassMismatch,
    #[error("Error deserializing PlumBody content")]
    DeserializationError,
    #[error("Failed to load Plum from Datahost")]
    FailedToLoadPlum,
}
