use crate::{DatahostStorageError, PathStateError};
use idp_proto::PlumHeadSeal;

#[derive(Debug, thiserror::Error)]
pub enum BranchError {
    #[error("BranchNode ancestor Plum {0} was missing on this Datahost")]
    BranchNodeAncestorPlumIsMissing(PlumHeadSeal),
    #[error("BranchNode Plum {0} was expected to already exist on this Datahost")]
    BranchNodePlumMustAlreadyExist(PlumHeadSeal),
    #[error(transparent)]
    DatahostStorageError(DatahostStorageError),
    #[error("Branch fast-forward operation expected new branch head ({new_branch_head}) to be a descendant of current branch head ({current_branch_head})")]
    FastForwardExpectedDescendant {
        current_branch_head: PlumHeadSeal,
        new_branch_head: PlumHeadSeal,
    },
    #[error("Branch fork-history operation expected new branch head ({new_branch_head}) and current branch head ({current_branch_head}) to have a common ancestor")]
    ForkHistoryExpectedCommonAncestor {
        current_branch_head: PlumHeadSeal,
        new_branch_head: PlumHeadSeal,
    },
    #[error("Internal error: {description}")]
    InternalError { description: String },
    #[error("Malformed request; {description}")]
    MalformedRequest { description: String },
    #[error("Max ancestor depth reached searching for closest common ancestor of {lhs} and {rhs}")]
    MaxAncestorDepthReached {
        lhs: PlumHeadSeal,
        rhs: PlumHeadSeal,
    },
    #[error(transparent)]
    PathStateError(PathStateError),
    #[error("Plum {plum_head_seal} was expected to be a BranchNode, but {description}")]
    PlumIsNotABranchNode {
        plum_head_seal: PlumHeadSeal,
        description: String,
    },
    #[error("Branch rewind operation expected new branch head ({new_branch_head}) to be an ancestor of current branch head ({current_branch_head})")]
    RewindExpectedAncestor {
        current_branch_head: PlumHeadSeal,
        new_branch_head: PlumHeadSeal,
    },
    #[error("Branch total-rewrite operation expected new branch head ({new_branch_head}) and current branch head ({current_branch_head}) to have no common ancestor")]
    TotalRewriteExpectedNoCommonAncestor {
        current_branch_head: PlumHeadSeal,
        new_branch_head: PlumHeadSeal,
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
            BranchError::BranchNodeAncestorPlumIsMissing(_) => {
                tonic::Status::failed_precondition(branch_error.to_string())
            }
            BranchError::BranchNodePlumMustAlreadyExist(_) => {
                tonic::Status::failed_precondition(branch_error.to_string())
            }
            BranchError::DatahostStorageError(datahost_storage_error) => {
                return datahost_storage_error.into();
            }
            BranchError::FastForwardExpectedDescendant { .. } => {
                tonic::Status::invalid_argument(branch_error.to_string())
            }
            BranchError::ForkHistoryExpectedCommonAncestor { .. } => {
                tonic::Status::invalid_argument(branch_error.to_string())
            }
            BranchError::InternalError { .. } => tonic::Status::internal(branch_error.to_string()),
            BranchError::MalformedRequest { .. } => {
                tonic::Status::invalid_argument(branch_error.to_string())
            }
            BranchError::MaxAncestorDepthReached { .. } => {
                tonic::Status::internal(branch_error.to_string())
            }
            BranchError::PathStateError(path_state_error) => {
                return path_state_error.into();
            }
            BranchError::PlumIsNotABranchNode { .. } => {
                tonic::Status::failed_precondition(branch_error.to_string())
            }
            BranchError::RewindExpectedAncestor { .. } => {
                tonic::Status::invalid_argument(branch_error.to_string())
            }
            BranchError::TotalRewriteExpectedNoCommonAncestor { .. } => {
                tonic::Status::invalid_argument(branch_error.to_string())
            }
        }
    }
}
