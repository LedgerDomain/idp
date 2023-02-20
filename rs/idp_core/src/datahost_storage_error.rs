use idp_proto::{PlumBodySeal, PlumHeadSeal, PlumRelationsSeal, PlumVerifyError};

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
    #[error("PlumHead {0} not found")]
    PlumHeadNotFound(PlumHeadSeal),
    #[error("PlumRelations {0} not found")]
    PlumRelationsNotFound(PlumRelationsSeal),
    #[error("PlumBody {0} not found")]
    PlumBodyNotFound(PlumBodySeal),
    #[error(transparent)]
    PlumHeadVerifyError(PlumVerifyError),
    #[cfg(feature = "sqlx-error")]
    #[error(transparent)]
    SqlxError(sqlx::Error),
}
