use idp_proto::{Path, PlumHeadSeal};

#[derive(Debug, derive_more::From, thiserror::Error)]
pub enum PathStateError {
    #[error("Invalid path `{path}` -- {reason}")]
    InvalidPath { path: Path, reason: String },
    #[error("Path `{0}` is expected not to exist yet")]
    PathAlreadyExists(Path),
    #[error("Plum {0} is expected to already exist")]
    PlumMustAlreadyExist(PlumHeadSeal),
}

#[cfg(feature = "tonic")]
impl From<PathStateError> for tonic::Status {
    fn from(path_state_error: PathStateError) -> Self {
        // It would be better to serialize the BranchError and deserialize it on the IDPClient side.
        match path_state_error {
            PathStateError::InvalidPath { .. } => {
                tonic::Status::invalid_argument(path_state_error.to_string())
            }
            PathStateError::PathAlreadyExists(_) => {
                tonic::Status::already_exists(path_state_error.to_string())
            }
            PathStateError::PlumMustAlreadyExist(_) => {
                tonic::Status::failed_precondition(path_state_error.to_string())
            }
        }
    }
}
