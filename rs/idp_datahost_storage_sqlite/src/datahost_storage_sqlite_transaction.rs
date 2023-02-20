use futures::{future::BoxFuture, stream::BoxStream};
use idp_core::{DatahostStorageError, DatahostStorageTransaction};

#[derive(
    Debug, derive_more::Deref, derive_more::DerefMut, derive_more::From, derive_more::Into,
)]
pub struct DatahostStorageSQLiteTransaction(sqlx::Transaction<'static, sqlx::Sqlite>);

#[async_trait::async_trait]
impl DatahostStorageTransaction for DatahostStorageSQLiteTransaction {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self as &mut dyn std::any::Any
    }
    async fn commit(self: Box<Self>) -> Result<(), DatahostStorageError> {
        Ok((*self).0.commit().await?)
    }
    async fn rollback(self: Box<Self>) -> Result<(), DatahostStorageError> {
        Ok((*self).0.rollback().await?)
    }
}

pub fn sqlite_transaction_mut<'a>(
    transaction: &'a mut dyn DatahostStorageTransaction,
) -> &'a mut DatahostStorageSQLiteTransaction {
    idp_core::downcast_transaction_mut(transaction)
}

impl<'c> sqlx::Executor<'c> for &'c mut DatahostStorageSQLiteTransaction {
    type Database = sqlx::Sqlite;

    fn describe<'e, 'q: 'e>(
        self,
        sql: &'q str,
    ) -> BoxFuture<'e, Result<sqlx::Describe<Self::Database>, sqlx::Error>>
    where
        'c: 'e,
    {
        self.0.describe(sql)
    }
    fn execute<'e, 'q: 'e, E: 'q>(
        self,
        query: E,
    ) -> BoxFuture<'e, Result<<Self::Database as sqlx::Database>::QueryResult, sqlx::Error>>
    where
        'c: 'e,
        E: sqlx::Execute<'q, Self::Database>,
    {
        self.0.execute(query)
    }
    fn execute_many<'e, 'q: 'e, E: 'q>(
        self,
        query: E,
    ) -> BoxStream<'e, Result<<Self::Database as sqlx::Database>::QueryResult, sqlx::Error>>
    where
        'c: 'e,
        E: sqlx::Execute<'q, Self::Database>,
    {
        self.0.execute_many(query)
    }
    fn fetch<'e, 'q: 'e, E: 'q>(
        self,
        query: E,
    ) -> BoxStream<'e, Result<<Self::Database as sqlx::Database>::Row, sqlx::Error>>
    where
        'c: 'e,
        E: sqlx::Execute<'q, Self::Database>,
    {
        self.0.fetch(query)
    }
    fn fetch_all<'e, 'q: 'e, E: 'q>(
        self,
        query: E,
    ) -> BoxFuture<'e, Result<Vec<<Self::Database as sqlx::Database>::Row>, sqlx::Error>>
    where
        'c: 'e,
        E: sqlx::Execute<'q, Self::Database>,
    {
        self.0.fetch_all(query)
    }
    fn fetch_many<'e, 'q: 'e, E: 'q>(
        self,
        query: E,
    ) -> BoxStream<
        'e,
        Result<
            sqlx::Either<
                <Self::Database as sqlx::Database>::QueryResult,
                <Self::Database as sqlx::Database>::Row,
            >,
            sqlx::Error,
        >,
    >
    where
        'c: 'e,
        E: sqlx::Execute<'q, Self::Database>,
    {
        self.0.fetch_many(query)
    }
    fn fetch_one<'e, 'q: 'e, E: 'q>(
        self,
        query: E,
    ) -> BoxFuture<'e, Result<<Self::Database as sqlx::Database>::Row, sqlx::Error>>
    where
        'c: 'e,
        E: sqlx::Execute<'q, Self::Database>,
    {
        self.0.fetch_one(query)
    }
    fn fetch_optional<'e, 'q: 'e, E: 'q>(
        self,
        query: E,
    ) -> BoxFuture<'e, Result<Option<<Self::Database as sqlx::Database>::Row>, sqlx::Error>>
    where
        'c: 'e,
        E: sqlx::Execute<'q, Self::Database>,
    {
        self.0.fetch_optional(query)
    }
    fn prepare<'e, 'q: 'e>(
        self,
        query: &'q str,
    ) -> BoxFuture<
        'e,
        Result<<Self::Database as sqlx::database::HasStatement<'q>>::Statement, sqlx::Error>,
    >
    where
        'c: 'e,
    {
        self.0.prepare(query)
    }
    fn prepare_with<'e, 'q: 'e>(
        self,
        sql: &'q str,
        parameters: &'e [<Self::Database as sqlx::Database>::TypeInfo],
    ) -> BoxFuture<
        'e,
        Result<<Self::Database as sqlx::database::HasStatement<'q>>::Statement, sqlx::Error>,
    >
    where
        'c: 'e,
    {
        self.0.prepare_with(sql, parameters)
    }
}
