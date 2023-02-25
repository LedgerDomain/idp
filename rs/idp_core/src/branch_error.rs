use crate::{DatahostStorageError, PathStateError};
use idp_proto::PlumHeadSeal;

#[derive(Debug, thiserror::Error)]
pub enum BranchError {
    #[error("BranchNode Plum {0} was expected to already exist on this Datahost")]
    BranchNodePlumMustAlreadyExist(PlumHeadSeal),
    #[error(transparent)]
    DatahostStorageError(DatahostStorageError),
    #[error("Internal error: {description}")]
    InternalError { description: String },
    #[error("Malformed request; {description}")]
    MalformedRequest { description: String },
    #[error(transparent)]
    PathStateError(PathStateError),
    #[error("Plum {plum_head_seal} was expected to be a BranchNode, but {description}")]
    PlumIsNotABranchNode {
        plum_head_seal: PlumHeadSeal,
        description: String,
    },
}

impl From<DatahostStorageError> for BranchError {
    fn from(datahost_storage_error: DatahostStorageError) -> Self {
        BranchError::DatahostStorageError(datahost_storage_error)
    }
}

impl From<PathStateError> for BranchError {
    fn from(path_state_error: PathStateError) -> Self {
        BranchError::PathStateError(path_state_error)
    }
}

#[cfg(feature = "tonic")]
impl From<BranchError> for tonic::Status {
    fn from(branch_error: BranchError) -> Self {
        // It would be better to serialize the BranchError and deserialize it on the IDPClient side.
        match branch_error {
            BranchError::BranchNodePlumMustAlreadyExist(_) => {
                tonic::Status::failed_precondition(branch_error.to_string())
            }
            BranchError::DatahostStorageError(datahost_storage_error) => {
                return datahost_storage_error.into();
            }
            BranchError::InternalError { .. } => tonic::Status::internal(branch_error.to_string()),
            BranchError::MalformedRequest { .. } => {
                tonic::Status::invalid_argument(branch_error.to_string())
            }
            BranchError::PathStateError(path_state_error) => {
                return path_state_error.into();
            }
            BranchError::PlumIsNotABranchNode { .. } => {
                tonic::Status::failed_precondition(branch_error.to_string())
            }
        }
    }
}
