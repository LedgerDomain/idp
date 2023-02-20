use crate::DatahostStorageError;

#[async_trait::async_trait]
pub trait DatahostStorageTransaction: Send + 'static {
    /// See "Wait, why do we need as_any?":
    /// https://stackoverflow.com/questions/33687447/how-to-get-a-reference-to-a-concrete-type-from-a-trait-object
    /// Alternate solution: https://crates.io/crates/mopa
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    async fn commit(self: Box<Self>) -> Result<(), DatahostStorageError>;
    async fn rollback(self: Box<Self>) -> Result<(), DatahostStorageError>;
}

pub fn downcast_transaction_mut<'a, T: 'static>(
    transaction: &'a mut dyn DatahostStorageTransaction,
) -> &'a mut T {
    transaction.as_any_mut().downcast_mut().unwrap()
}
