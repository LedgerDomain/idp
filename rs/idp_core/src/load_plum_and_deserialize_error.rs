#[derive(Clone, Copy, Debug, thiserror::Error)]
pub enum LoadPlumAndDeserializeError {
    #[error("ContentType mismatch")]
    ContentTypeMismatch,
    #[error("Error deserializing PlumBody content")]
    DeserializationError,
    #[error("Failed to load Plum from Datahost")]
    FailedToLoadPlum,
}
