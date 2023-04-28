use idp_proto::{
    Path, PlumBodySeal, PlumHeadSeal, PlumMetadataSeal, PlumRelationsSeal, PlumVerifyError,
};

#[derive(Debug, derive_more::From, thiserror::Error)]
pub enum DatahostStorageError {
    #[error(transparent)]
    Generic(Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("Read an invalid value from table {table_name} column {column_name}")]
    InvalidValueInDB {
        table_name: &'static str,
        column_name: &'static str,
        reason: String,
    },
    // #[error("Path {0} already exists")]
    // PathAlreadyExists(Path),
    #[error("Path {0} not found")]
    PathNotFound(Path),
    #[error("PlumHead {0} not found")]
    PlumHeadNotFound(PlumHeadSeal),
    #[error("PlumMetadata {0} not found")]
    PlumMetadataNotFound(PlumMetadataSeal),
    #[error("PlumRelations {0} not found")]
    PlumRelationsNotFound(PlumRelationsSeal),
    #[error("PlumBody {0} not found")]
    PlumBodyNotFound(PlumBodySeal),
    #[error(transparent)]
    PlumVerifyError(PlumVerifyError),
    #[cfg(feature = "sqlx-error")]
    #[error(transparent)]
    SqlxError(sqlx::Error),
}

#[cfg(feature = "tonic")]
impl From<DatahostStorageError> for tonic::Status {
    fn from(datahost_storage_error: DatahostStorageError) -> Self {
        // It would be better to serialize the DatahostStorageError and deserialize it on the IDPClient side.
        let code = match &datahost_storage_error {
            DatahostStorageError::Generic(_) => tonic::Code::Unknown,
            DatahostStorageError::InvalidValueInDB { .. } => tonic::Code::DataLoss,
            // DatahostStorageError::PathAlreadyExists(_) => tonic::Code::AlreadyExists,
            DatahostStorageError::PathNotFound(_) => tonic::Code::NotFound,
            DatahostStorageError::PlumHeadNotFound(_) => tonic::Code::NotFound,
            DatahostStorageError::PlumMetadataNotFound(_) => tonic::Code::NotFound,
            DatahostStorageError::PlumRelationsNotFound(_) => tonic::Code::NotFound,
            DatahostStorageError::PlumBodyNotFound(_) => tonic::Code::NotFound,
            DatahostStorageError::PlumVerifyError(_) => tonic::Code::InvalidArgument,
            #[cfg(feature = "sqlx-error")]
            DatahostStorageError::SqlxError(_) => tonic::Code::Internal,
        };
        tonic::Status::new(code, datahost_storage_error.to_string())
    }
}
