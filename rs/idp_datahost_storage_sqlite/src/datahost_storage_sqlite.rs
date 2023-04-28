use crate::{sqlite_transaction_mut, DatahostStorageSQLiteTransaction};
use idp_core::{DatahostStorage, DatahostStorageError, DatahostStorageTransaction};
use idp_proto::{
    Content, ContentClass, ContentEncoding, ContentFormat, ContentMetadata, Nonce, Path, PathState,
    PlumBody, PlumBodySeal, PlumHead, PlumHeadSeal, PlumMetadata, PlumMetadataSeal,
    PlumRelationFlags, PlumRelationFlagsMapping, PlumRelations, PlumRelationsSeal, Seal, Sha256Sum,
    UnixNanoseconds,
};

pub struct DatahostStorageSQLite {
    pool: sqlx::SqlitePool,
}

impl DatahostStorageSQLite {
    /// Connect to the SQLite DB at the given URL.  Note that the URL ":memory:" or "sqlite::memory:"
    /// opens an ephemeral, in-memory DB.
    pub async fn connect_and_run_migrations(url: &str) -> Result<Self, sqlx::Error> {
        let pool = Self::pool_connect(url).await?;
        sqlx::migrate!().run(&pool).await?;
        Ok(Self { pool })
    }
    /// Convenience method for connecting to an ephemeral, in-memory DB.
    pub async fn new_in_memory() -> Result<Self, sqlx::Error> {
        Self::connect_and_run_migrations(":memory:").await
    }

    /// This sets the SQL statement logging to sane levels -- trace for statements, and warn for slow statements.
    async fn pool_connect(url: &str) -> Result<sqlx::SqlitePool, sqlx::Error> {
        // Reference for changing logging level for SQL statements:
        // https://github.com/launchbadge/sqlx/discussions/1056
        use std::str::FromStr;
        let mut connect_options = sqlx::sqlite::SqliteConnectOptions::from_str(url)?;
        use sqlx::ConnectOptions;
        connect_options
            .log_statements(log::LevelFilter::Trace)
            .log_slow_statements(log::LevelFilter::Warn, std::time::Duration::from_secs(1));
        sqlx::sqlite::SqlitePoolOptions::new()
            .connect_with(connect_options)
            .await
    }
}

#[async_trait::async_trait]
impl DatahostStorage for DatahostStorageSQLite {
    async fn begin_transaction(
        &self,
    ) -> Result<Box<dyn DatahostStorageTransaction>, DatahostStorageError> {
        Ok(Box::new(DatahostStorageSQLiteTransaction::from(
            self.pool.begin().await?,
        )))
    }

    async fn has_plum_head(
        &self,
        transaction: &mut dyn DatahostStorageTransaction,
        plum_head_seal: &PlumHeadSeal,
    ) -> Result<bool, DatahostStorageError> {
        let sqlite_transaction = sqlite_transaction_mut(transaction);
        let value = sqlx::query!(
            "SELECT EXISTS(SELECT 1 FROM plum_heads WHERE plum_head_seal = $1) AS value",
            plum_head_seal.value.sha256sum.value
        )
        .fetch_one(sqlite_transaction)
        .await?
        .value;
        Ok(value != 0)
    }
    async fn has_plum_metadata(
        &self,
        transaction: &mut dyn DatahostStorageTransaction,
        plum_metadata_seal: &PlumMetadataSeal,
    ) -> Result<bool, DatahostStorageError> {
        let sqlite_transaction = sqlite_transaction_mut(transaction);
        let value = sqlx::query!(
            "SELECT EXISTS(SELECT 1 FROM plum_metadatas WHERE plum_metadata_seal = $1) AS value",
            plum_metadata_seal.value.sha256sum.value
        )
        .fetch_one(sqlite_transaction)
        .await?
        .value;
        Ok(value != 0)
    }
    async fn has_plum_relations(
        &self,
        transaction: &mut dyn DatahostStorageTransaction,
        plum_relations_seal: &PlumRelationsSeal,
    ) -> Result<bool, DatahostStorageError> {
        let sqlite_transaction = sqlite_transaction_mut(transaction);
        let value = sqlx::query!(
            "SELECT EXISTS(SELECT 1 FROM plum_relations WHERE plum_relations_seal = $1) AS value",
            plum_relations_seal.value.sha256sum.value
        )
        .fetch_one(sqlite_transaction)
        .await?
        .value;
        Ok(value != 0)
    }
    async fn has_plum_body(
        &self,
        transaction: &mut dyn DatahostStorageTransaction,
        plum_body_seal: &PlumBodySeal,
    ) -> Result<bool, DatahostStorageError> {
        let sqlite_transaction = sqlite_transaction_mut(transaction);
        let value = sqlx::query!(
            r#"SELECT EXISTS(SELECT 1 FROM plum_bodies WHERE plum_body_seal = $1) AS value"#,
            plum_body_seal.value.sha256sum.value
        )
        .fetch_one(sqlite_transaction)
        .await?
        .value;
        Ok(value != 0)
    }

    async fn store_plum_head(
        &self,
        transaction: &mut dyn DatahostStorageTransaction,
        plum_head: &PlumHead,
    ) -> Result<PlumHeadSeal, DatahostStorageError> {
        let plum_head_seal = PlumHeadSeal::from(plum_head);
        log::debug!(
            "store_plum_head; storing plum_head with seal: {}",
            plum_head_seal
        );

        // TEMP HACK
        if self
            .has_plum_head(&mut *transaction, &plum_head_seal)
            .await?
        {
            // Can early out in this case
            log::debug!(
                "store_plum_head; already had plum_head with seal: {}",
                plum_head_seal
            );
            return Ok(plum_head_seal);
        }

        let sqlite_transaction = sqlite_transaction_mut(transaction);

        let now = UnixNanoseconds::now();

        // Due to https://github.com/launchbadge/sqlx/issues/1430 it seems that these temps are unavoidable.
        let plum_head_nonce_o = plum_head
            .plum_head_nonce_o
            .as_ref()
            .map(|plum_head_nonce| &plum_head_nonce.value);
        // Ignore collision.  The PlumHeadSeal being identical should guarantee that the plum_heads row is
        // identical except for the plum_heads_rowid and row_inserted_at.  However, it might be good to add
        // a check upon collision that the row is actually identical.
        let _plum_heads_rowid = sqlx::query!(
            r#"INSERT INTO plum_heads (
                row_inserted_at,
                plum_head_seal,
                plum_head_nonce_o,
                plum_metadata_seal,
                plum_relations_seal,
                plum_body_seal
            ) VALUES ($1, $2, $3, $4, $5, $6)
            -- ON CONFLICT(plum_head_seal) DO NOTHING
            RETURNING plum_heads_rowid
            --;
            -- NOTE: This doesn't work
            --SELECT last_insert_rowid() AS plum_heads_rowid"#,
            now.value,
            plum_head_seal.value.sha256sum.value,
            plum_head_nonce_o,
            plum_head.plum_metadata_seal.value.sha256sum.value,
            plum_head.plum_relations_seal.value.sha256sum.value,
            plum_head.plum_body_seal.value.sha256sum.value,
        )
        .fetch_one(sqlite_transaction)
        .await?
        .plum_heads_rowid;

        Ok(plum_head_seal)
    }
    async fn store_plum_metadata(
        &self,
        transaction: &mut dyn DatahostStorageTransaction,
        plum_metadata: &PlumMetadata,
    ) -> Result<PlumMetadataSeal, DatahostStorageError> {
        let plum_metadata_seal = PlumMetadataSeal::from(plum_metadata);
        log::debug!(
            "store_plum_metadata; storing plum_metadata with seal: {}",
            plum_metadata_seal
        );

        // TEMP HACK
        if self
            .has_plum_metadata(&mut *transaction, &plum_metadata_seal)
            .await?
        {
            // Can early out in this case
            log::debug!(
                "store_plum_metadata; already had plum_metadata with seal: {}",
                plum_metadata_seal
            );
            return Ok(plum_metadata_seal);
        }

        let sqlite_transaction = sqlite_transaction_mut(transaction);

        let now = UnixNanoseconds::now();

        // Due to https://github.com/launchbadge/sqlx/issues/1430 it seems that these temps are unavoidable.
        let plum_metadata_nonce_o = plum_metadata
            .plum_metadata_nonce_o
            .as_ref()
            .map(|plum_metadata_nonce| &plum_metadata_nonce.value);

        let plum_created_at_o = plum_metadata
            .plum_created_at_o
            .map(|created_at| created_at.value);

        let plum_body_content_length_o = if let Some(plum_body_content_metadata) =
            plum_metadata.plum_body_content_metadata_o.as_ref()
        {
            // SQLite doesn't support u64, so we have to check for overflow.
            if plum_body_content_metadata.content_length > (i64::MAX as u64) {
                panic!("plum_body_content_length too large (exceeds max of i64) in PlumMetadata");
            }
            Some(plum_body_content_metadata.content_length as i64)
        } else {
            None
        };
        let plum_body_content_class_o = plum_metadata
            .plum_body_content_metadata_o
            .as_ref()
            .map(|x| x.content_class.as_str());
        let plum_body_content_format_o = plum_metadata
            .plum_body_content_metadata_o
            .as_ref()
            .map(|x| x.content_format.as_str());
        let plum_body_content_encoding_o = plum_metadata
            .plum_body_content_metadata_o
            .as_ref()
            .map(|x| x.content_encoding.as_str());

        let additional_content_length_o = if let Some(additional_content) =
            plum_metadata.additional_content_o.as_ref()
        {
            // SQLite doesn't support u64, so we have to check for overflow.
            if additional_content.content_metadata.content_length > (i64::MAX as u64) {
                panic!(
                        "content_length too large (exceeds max of i64) in PlumMetadata additional_content"
                    );
            }
            Some(additional_content.content_metadata.content_length as i64)
        } else {
            None
        };
        let additional_content_class_o = plum_metadata
            .additional_content_o
            .as_ref()
            .map(|x| x.content_metadata.content_class.as_str());
        let additional_content_format_o = plum_metadata
            .additional_content_o
            .as_ref()
            .map(|x| x.content_metadata.content_format.as_str());
        let additional_content_encoding_o = plum_metadata
            .additional_content_o
            .as_ref()
            .map(|x| x.content_metadata.content_encoding.as_str());
        let additional_content_byte_vo = plum_metadata
            .additional_content_o
            .as_ref()
            .map(|x| x.content_byte_v.as_slice());

        // Ignore collision.  The PlumMetadataSeal being identical should guarantee that the plum_metadatas row is
        // identical except for the plum_metadatas_rowid and row_inserted_at.  However, it might be good to add
        // a check upon collision that the row is actually identical.
        let _plum_metadatas_rowid = sqlx::query!(
            r#"INSERT INTO plum_metadatas (
                row_inserted_at,
                plum_metadata_seal,
                plum_metadata_nonce_o,
                plum_created_at_o,
                plum_body_content_length_o,
                plum_body_content_class_o,
                plum_body_content_format_o,
                plum_body_content_encoding_o,
                additional_content_length_o,
                additional_content_class_o,
                additional_content_format_o,
                additional_content_encoding_o,
                additional_content_byte_vo
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            -- ON CONFLICT(plum_metadata_seal) DO NOTHING
            RETURNING plum_metadatas_rowid
            --;
            -- NOTE: This doesn't work
            --SELECT last_insert_rowid() AS plum_metadatas_rowid"#,
            now.value,
            plum_metadata_seal.value.sha256sum.value,
            plum_metadata_nonce_o,
            plum_created_at_o,
            plum_body_content_length_o,
            plum_body_content_class_o,
            plum_body_content_format_o,
            plum_body_content_encoding_o,
            additional_content_length_o,
            additional_content_class_o,
            additional_content_format_o,
            additional_content_encoding_o,
            additional_content_byte_vo,
        )
        .fetch_one(sqlite_transaction)
        .await?
        .plum_metadatas_rowid;

        Ok(plum_metadata_seal)
    }
    async fn store_plum_relations(
        &self,
        transaction: &mut dyn DatahostStorageTransaction,
        plum_relations: &PlumRelations,
    ) -> Result<PlumRelationsSeal, DatahostStorageError> {
        let plum_relations_seal = PlumRelationsSeal::from(plum_relations);
        log::debug!(
            "store_plum_relations; storing plum_relations with seal: {}",
            plum_relations_seal
        );

        // TEMP HACK -- this should be possible to handle with the ON CONFLICT below, but
        // I'm having trouble with that.
        if self
            .has_plum_relations(&mut *transaction, &plum_relations_seal)
            .await?
        {
            // Can early out in this case.
            log::debug!(
                "store_plum_relations; already had plum_relations with seal: {}",
                plum_relations_seal
            );
            return Ok(plum_relations_seal);
        }

        let sqlite_transaction = sqlite_transaction_mut(transaction);

        let now = UnixNanoseconds::now();

        // Due to https://github.com/launchbadge/sqlx/issues/1430 it seems that these temps are unavoidable.
        let plum_relations_nonce_o = plum_relations
            .plum_relations_nonce_o
            .as_ref()
            .map(|plum_relations_nonce| &plum_relations_nonce.value);
        // Ignore collision.  The PlumRelationsSeal being identical should guarantee that the plum_relationss row is
        // identical except for the plum_relations_rowid and row_inserted_at.  However, it might be good to add
        // a check upon collision that the row is actually identical.

        // let plum_relations_rowid_o = sqlx::query!(
        //     r#"INSERT INTO plum_relations (
        //         row_inserted_at,
        //         plum_relations_seal,
        //         plum_relations_nonce_o,
        //         source_plum_body_seal
        //     ) VALUES ($1, $2, $3, $4)
        //     ON CONFLICT(plum_relations_seal) DO NOTHING
        //     RETURNING plum_relations_rowid;
        //     --;
        //     --SELECT last_insert_rowid() AS plum_relations_rowid"#,
        //     now.value,
        //     plum_relations_seal.value.sha256sum.value,
        //     plum_relations_nonce_o,
        //     plum_relations.source_plum_body_seal.value.sha256sum.value,
        // )
        // .fetch_one(&mut *sqlite_transaction)
        // .await?
        // .plum_relations_rowid;
        // // .fetch_optional(&mut *sqlite_transaction)
        // // .await?
        // // .map(|record| record.plum_relations_rowid);

        let plum_relations_rowid = sqlx::query!(
            r#"INSERT INTO plum_relations (
                row_inserted_at,
                plum_relations_seal,
                plum_relations_nonce_o,
                source_plum_body_seal
            ) VALUES ($1, $2, $3, $4)
            RETURNING plum_relations_rowid"#,
            now.value,
            plum_relations_seal.value.sha256sum.value,
            plum_relations_nonce_o,
            plum_relations.source_plum_body_seal.value.sha256sum.value,
        )
        .fetch_one(&mut *sqlite_transaction)
        .await?
        .plum_relations_rowid;

        // if plum_relations_rowid_o.is_none() {
        //     // This indicates that the "ON CONFLICT DO NOTHING" clause triggered, so there's no
        //     // need to add the relation mappings.
        //     return Ok(plum_relations_seal);
        // }
        // let plum_relations_rowid = plum_relations_rowid_o.unwrap();

        // TODO: Figure out how to do this efficiently (i.e. batch insert perhaps with appropriate chunking)
        for plum_relation_flags_mapping in plum_relations.plum_relation_flags_mapping_v.iter() {
            sqlx::query!(
                r#"INSERT INTO plum_relation_mappings (
                    plum_relations_rowid,
                    target_plum_head_seal,
                    plum_relation_flags
                ) VALUES ($1, $2, $3);"#,
                plum_relations_rowid,
                plum_relation_flags_mapping
                    .target_plum_head_seal
                    .value
                    .sha256sum
                    .value,
                plum_relation_flags_mapping.plum_relation_flags_raw.value,
            )
            .execute(&mut *sqlite_transaction)
            .await?;
        }

        Ok(plum_relations_seal)
    }
    async fn store_plum_body(
        &self,
        transaction: &mut dyn DatahostStorageTransaction,
        plum_body: &PlumBody,
    ) -> Result<PlumBodySeal, DatahostStorageError> {
        let plum_body_seal = PlumBodySeal::from(plum_body);
        log::debug!(
            "store_plum_body; storing plum_body with seal: {:?}",
            plum_body_seal
        );

        // TEMP HACK
        if self
            .has_plum_body(&mut *transaction, &plum_body_seal)
            .await?
        {
            // Can early out in this case.
            log::debug!(
                "store_plum_body; already had plum_body with seal: {}",
                plum_body_seal
            );
            return Ok(plum_body_seal);
        }

        let sqlite_transaction = sqlite_transaction_mut(transaction);

        // NOTE: Having the whole PlumBody (with its contents) is not scalable.
        // Probably need to at least chunk the bodies and use a merkle tree to compute the hash,
        // and probably actually write them to disk.
        let now = UnixNanoseconds::now();

        // Due to https://github.com/launchbadge/sqlx/issues/1430 it seems that these temps are unavoidable.
        let plum_body_nonce_o = plum_body
            .plum_body_nonce_o
            .as_ref()
            .map(|plum_body_nonce| &plum_body_nonce.value);
        // SQLite doesn't support u64, so we have to check for overflow.
        let plum_body_content_length =
            if plum_body.plum_body_content.content_metadata.content_length > (i64::MAX as u64) {
                panic!("plum_body_content_length too large (exceeds max of i64) in PlumMetadata");
            } else {
                plum_body.plum_body_content.content_metadata.content_length as i64
            };
        let plum_body_content_class = plum_body
            .plum_body_content
            .content_metadata
            .content_class
            .as_str();
        let plum_body_content_format = plum_body
            .plum_body_content
            .content_metadata
            .content_format
            .as_str();
        let plum_body_content_encoding = plum_body
            .plum_body_content
            .content_metadata
            .content_encoding
            .as_str();

        // Ignore collision.  The PlumBodySeal being identical should guarantee that the plum_bodies row is
        // identical except for the plum_bodies_rowid and row_inserted_at.  However, it might be good to add
        // a check upon collision that the row is actually identical.
        let _plum_bodies_rowid = sqlx::query!(
            r#"INSERT INTO plum_bodies (
                row_inserted_at,
                plum_body_seal,
                plum_body_nonce_o,
                plum_body_content_length,
                plum_body_content_class,
                plum_body_content_format,
                plum_body_content_encoding,
                plum_body_content_byte_v
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            --ON CONFLICT(plum_body_seal) DO NOTHING
            RETURNING plum_bodies_rowid
            --;
            --SELECT last_insert_rowid() AS plum_bodies_rowid"#,
            now.value,
            plum_body_seal.value.sha256sum.value,
            plum_body_nonce_o,
            plum_body_content_length,
            plum_body_content_class,
            plum_body_content_format,
            plum_body_content_encoding,
            plum_body.plum_body_content.content_byte_v,
        )
        .fetch_one(sqlite_transaction)
        .await?
        .plum_bodies_rowid;

        Ok(plum_body_seal)
    }

    async fn load_option_plum_head(
        &self,
        transaction: &mut dyn DatahostStorageTransaction,
        plum_head_seal: &PlumHeadSeal,
    ) -> Result<Option<PlumHead>, DatahostStorageError> {
        let sqlite_transaction = sqlite_transaction_mut(transaction);

        let plum_heads_row_r = sqlx::query!(
            r#"SELECT
                plum_head_nonce_o,
                plum_metadata_seal,
                plum_relations_seal,
                plum_body_seal
            FROM plum_heads
            WHERE plum_head_seal = $1"#,
            plum_head_seal.value.sha256sum.value
        )
        .fetch_one(sqlite_transaction)
        .await;

        match plum_heads_row_r {
            Ok(plum_heads_row) => Ok(Some({
                PlumHead {
                    plum_head_nonce_o: plum_heads_row.plum_head_nonce_o.map(Nonce::from),
                    plum_metadata_seal: PlumMetadataSeal::from(Seal::from(Sha256Sum::from(
                        plum_heads_row.plum_metadata_seal,
                    ))),
                    plum_relations_seal: PlumRelationsSeal::from(Seal::from(Sha256Sum::from(
                        plum_heads_row.plum_relations_seal,
                    ))),
                    plum_body_seal: PlumBodySeal::from(Seal::from(Sha256Sum::from(
                        plum_heads_row.plum_body_seal,
                    ))),
                }
            })),
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
    async fn load_option_plum_metadata(
        &self,
        transaction: &mut dyn DatahostStorageTransaction,
        plum_metadata_seal: &PlumMetadataSeal,
    ) -> Result<Option<PlumMetadata>, DatahostStorageError> {
        let sqlite_transaction = sqlite_transaction_mut(transaction);

        let plum_metadatas_row_r = sqlx::query!(
            r#"SELECT
                plum_metadata_nonce_o,
                plum_created_at_o,
                plum_body_content_length_o,
                plum_body_content_class_o,
                plum_body_content_format_o,
                plum_body_content_encoding_o,
                additional_content_length_o,
                additional_content_class_o,
                additional_content_format_o,
                additional_content_encoding_o,
                additional_content_byte_vo
            FROM plum_metadatas
            WHERE plum_metadata_seal = $1"#,
            plum_metadata_seal.value.sha256sum.value
        )
        .fetch_one(sqlite_transaction)
        .await;

        match plum_metadatas_row_r {
            Ok(plum_metadatas_row) => {
                // The plum_metadatas table has constraints where plum_body_content_length_o,
                // plum_body_content_class_o, plum_body_content_format_o, and plum_body_content_encoding_o
                // must all be NULL or all be non-NULL.
                let plum_body_content_metadata_o = match (
                    plum_metadatas_row.plum_body_content_length_o,
                    plum_metadatas_row.plum_body_content_class_o,
                    plum_metadatas_row.plum_body_content_format_o,
                    plum_metadatas_row.plum_body_content_encoding_o,
                ) {
                    (
                        Some(content_length),
                        Some(content_class),
                        Some(content_format),
                        Some(content_encoding),
                    ) => {
                        if content_length < 0 {
                            return Err(DatahostStorageError::InvalidValueInDB {
                                table_name: "plum_metadatas",
                                column_name: "plum_body_content_length_o",
                                reason: "column value was negative".to_string(),
                            });
                        }
                        Some(ContentMetadata {
                            content_length: content_length as u64,
                            content_class: ContentClass::from(content_class.to_string()),
                            content_format: ContentFormat::from(content_format.to_string()),
                            content_encoding: ContentEncoding::from(content_encoding.to_string()),
                        })
                    }
                    (None, None, None, None) => None,
                    _ => {
                        // Not sure if panic is the right way to go, but use it for now, so errors are detected early in development.
                        panic!("programmer error: DB constraint failure in plum_body_content_* columns of plum_metadatas table");
                    }
                };

                // The plum_metadatas table has constraints where additional_content_length_o,
                // additional_content_class_o, additional_content_format_o, additional_content_encoding_o,
                // and additional_content_byte_vo must all be NULL or all be non-NULL.
                let additional_content_o = match (
                    plum_metadatas_row.additional_content_length_o,
                    plum_metadatas_row.additional_content_class_o,
                    plum_metadatas_row.additional_content_format_o,
                    plum_metadatas_row.additional_content_encoding_o,
                    plum_metadatas_row.additional_content_byte_vo,
                ) {
                    (
                        Some(content_length),
                        Some(content_class),
                        Some(content_format),
                        Some(content_encoding),
                        Some(content_byte_v),
                    ) => {
                        if content_length < 0 {
                            return Err(DatahostStorageError::InvalidValueInDB {
                                table_name: "plum_metadatas",
                                column_name: "additional_content_length_o",
                                reason: "column value was negative".to_string(),
                            });
                        }
                        Some(Content {
                            content_metadata: ContentMetadata {
                                content_length: content_length as u64,
                                content_class: ContentClass::from(content_class.to_string()),
                                content_format: ContentFormat::from(content_format.to_string()),
                                content_encoding: ContentEncoding::from(
                                    content_encoding.to_string(),
                                ),
                            },
                            content_byte_v,
                        })
                    }
                    (None, None, None, None, None) => None,
                    _ => {
                        // Not sure if panic is the right way to go, but use it for now, so errors are detected early in development.
                        panic!("programmer error: DB constraint failure in additional_content_* columns of plum_metadatas table");
                    }
                };

                Ok(Some(PlumMetadata {
                    plum_metadata_nonce_o: plum_metadatas_row
                        .plum_metadata_nonce_o
                        .map(Nonce::from),
                    plum_created_at_o: plum_metadatas_row.plum_created_at_o.map(|x| x.into()),
                    plum_body_content_metadata_o,
                    additional_content_o,
                }))
            }
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
    async fn load_option_plum_relations(
        &self,
        transaction: &mut dyn DatahostStorageTransaction,
        plum_relations_seal: &PlumRelationsSeal,
    ) -> Result<Option<PlumRelations>, DatahostStorageError> {
        let sqlite_transaction = sqlite_transaction_mut(transaction);

        // TODO: No need to query the whole row, only need a few column values.
        let plum_relations_row_r = sqlx::query!(
            r#"SELECT
                plum_relations_rowid,
                row_inserted_at,
                plum_relations_seal,
                plum_relations_nonce_o,
                source_plum_body_seal
            FROM plum_relations
            WHERE plum_relations_seal = $1"#,
            plum_relations_seal.value.sha256sum.value
        )
        .fetch_one(&mut *sqlite_transaction)
        .await;

        let plum_relations_row = match plum_relations_row_r {
            Ok(plum_relations_row) => plum_relations_row,
            Err(sqlx::Error::RowNotFound) => {
                return Ok(None);
            }
            Err(e) => {
                return Err(e.into());
            }
        };

        let plum_relation_flags_mapping_v = sqlx::query!(
            r#"SELECT target_plum_head_seal, plum_relation_flags
            FROM plum_relation_mappings
            WHERE plum_relations_rowid = $1"#,
            plum_relations_row.plum_relations_rowid
        )
        .fetch_all(&mut *sqlite_transaction)
        .await?
        .into_iter()
        .map(
            |record| -> Result<PlumRelationFlagsMapping, DatahostStorageError> {
                if record.plum_relation_flags < 0 || record.plum_relation_flags > (u32::MAX as i64)
                {
                    return Err(DatahostStorageError::InvalidValueInDB {
                        table_name: "plum_relation_mappings",
                        column_name: "plum_relation_flags",
                        reason: "column value was outside of the range of u32".to_string(),
                    });
                }
                let plum_relation_flags = record.plum_relation_flags as u32;
                Ok(PlumRelationFlagsMapping {
                    target_plum_head_seal: PlumHeadSeal::from(Seal::from(Sha256Sum::from(
                        record.target_plum_head_seal,
                    ))),
                    plum_relation_flags_raw: PlumRelationFlags::try_from(plum_relation_flags)
                        .map_err(|e| DatahostStorageError::InvalidValueInDB {
                            table_name: "plum_relation_mappings",
                            column_name: "plum_relation_flags",
                            reason: e.to_string(),
                        })?
                        .into(),
                })
            },
        )
        .collect::<Result<Vec<PlumRelationFlagsMapping>, DatahostStorageError>>()?;

        Ok(Some(PlumRelations {
            plum_relations_nonce_o: plum_relations_row.plum_relations_nonce_o.map(Nonce::from),
            source_plum_body_seal: PlumBodySeal::from(Seal::from(Sha256Sum::from(
                plum_relations_row.source_plum_body_seal,
            ))),
            plum_relation_flags_mapping_v,
        }))
    }
    async fn load_option_plum_body(
        &self,
        transaction: &mut dyn DatahostStorageTransaction,
        plum_body_seal: &PlumBodySeal,
    ) -> Result<Option<PlumBody>, DatahostStorageError> {
        let sqlite_transaction = sqlite_transaction_mut(transaction);

        let record_r = sqlx::query!(
            r#"SELECT
                plum_body_nonce_o,
                plum_body_content_length,
                plum_body_content_class,
                plum_body_content_format,
                plum_body_content_encoding,
                plum_body_content_byte_v
            FROM plum_bodies
            WHERE plum_body_seal = $1"#,
            plum_body_seal.value.sha256sum.value
        )
        .fetch_one(&mut *sqlite_transaction)
        .await;

        match record_r {
            Ok(record) => {
                if record.plum_body_content_length < 0 {
                    return Err(DatahostStorageError::InvalidValueInDB {
                        table_name: "plum_bodies",
                        column_name: "plum_body_content_length",
                        reason: "column value was negative".to_string(),
                    });
                }
                Ok(Some(PlumBody {
                    plum_body_nonce_o: record.plum_body_nonce_o.map(Nonce::from),
                    plum_body_content: Content {
                        content_metadata: ContentMetadata {
                            content_length: record.plum_body_content_length as u64,
                            content_class: ContentClass::from(record.plum_body_content_class),
                            content_format: ContentFormat::from(record.plum_body_content_format),
                            content_encoding: ContentEncoding::from(
                                record.plum_body_content_encoding,
                            ),
                        },
                        content_byte_v: record.plum_body_content_byte_v,
                    },
                }))
            }
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn has_path_state(
        &self,
        transaction: &mut dyn DatahostStorageTransaction,
        path: &Path,
    ) -> Result<bool, DatahostStorageError> {
        let sqlite_transaction = sqlite_transaction_mut(transaction);
        // TODO: Would need to figure out how soft-deleted state is handled -- "has_path_state" really becomes
        // two parts, one is "has non-deleted path state" and the other is "has deleted path state".
        let value = sqlx::query!(
            "SELECT EXISTS(SELECT 1 FROM path_states WHERE path = $1) AS value",
            path.value
        )
        .fetch_one(sqlite_transaction)
        .await?
        .value;
        Ok(value != 0)
    }
    async fn load_option_path_state(
        &self,
        transaction: &mut dyn DatahostStorageTransaction,
        path: &Path,
    ) -> Result<Option<PathState>, DatahostStorageError> {
        let sqlite_transaction = sqlite_transaction_mut(transaction);

        let path_states_row_r = sqlx::query!(
            r#"SELECT
                path,
                current_state_plum_head_seal
            FROM path_states
            WHERE path = $1"#,
            path.value
        )
        .fetch_one(sqlite_transaction)
        .await;

        match path_states_row_r {
            Ok(path_states_row) => Ok(Some({
                PathState {
                    path: Path::from(path_states_row.path),
                    current_state_plum_head_seal: PlumHeadSeal::from(Seal::from(Sha256Sum::from(
                        path_states_row.current_state_plum_head_seal,
                    ))),
                }
            })),
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
    async fn insert_path_state(
        &self,
        transaction: &mut dyn DatahostStorageTransaction,
        path_state: &PathState,
    ) -> Result<(), DatahostStorageError> {
        let sqlite_transaction = sqlite_transaction_mut(transaction);
        let now = UnixNanoseconds::now();

        let query_result_r = sqlx::query!(
            r#"INSERT INTO path_states (
                row_inserted_at,
                row_updated_at,
                path,
                current_state_plum_head_seal
            ) VALUES ($1, $2, $3, $4)"#,
            now.value,
            now.value,
            path_state.path.value,
            path_state
                .current_state_plum_head_seal
                .value
                .sha256sum
                .value,
        )
        .execute(sqlite_transaction)
        .await;

        log::trace!(
            "DatahostStorageSQLite::insert_path_state; path_state: {:?}, query_result_r: {:?}",
            path_state,
            query_result_r
        );

        query_result_r?;

        // TODO: Handle collision -- not sure which error code is right.
        // match insert_r {
        //     Ok(insert) => Ok(()),
        //     Err(sqlx::Error::)

        // }

        Ok(())
    }
    async fn update_path_state(
        &self,
        transaction: &mut dyn DatahostStorageTransaction,
        path_state: &PathState,
    ) -> Result<(), DatahostStorageError> {
        let sqlite_transaction = sqlite_transaction_mut(transaction);
        let now = UnixNanoseconds::now();

        let query_result_r = sqlx::query!(
            r#"UPDATE path_states
            SET row_updated_at = $1,
                current_state_plum_head_seal = $2
            WHERE path = $3"#,
            now.value,
            path_state
                .current_state_plum_head_seal
                .value
                .sha256sum
                .value,
            path_state.path.value
        )
        .execute(sqlite_transaction)
        .await;

        log::trace!(
            "DatahostStorageSQLite::update_path_state; path_state: {:?}, query_result_r: {:?}",
            path_state,
            query_result_r
        );

        match query_result_r {
            Ok(query_result) => {
                if query_result.rows_affected() == 0 {
                    Err(DatahostStorageError::PathNotFound(path_state.path.clone()))
                } else {
                    Ok(())
                }
            }
            // NOTE: Not sure if this actually happens, or if it's always handled by above.
            Err(sqlx::Error::RowNotFound) => {
                Err(DatahostStorageError::PathNotFound(path_state.path.clone()))
            }
            Err(e) => Err(e.into()),
        }
    }
    async fn delete_path_state(
        &self,
        transaction: &mut dyn DatahostStorageTransaction,
        path: &Path,
    ) -> Result<(), DatahostStorageError> {
        let sqlite_transaction = sqlite_transaction_mut(transaction);
        // let now = UnixNanoseconds::now();

        // // Use soft deletes so that a non-owner can't resurrect the path and pass themselves off as the original.
        // sqlx::query!(
        //     r#"UPDATE path_states
        //     SET row_deleted_at = $1
        //     WHERE path = $3"#,
        //     now.value,
        //     path.value,
        // )
        // .fetch_one(sqlite_transaction)
        // .await?;

        let query_result_r = sqlx::query!(r#"DELETE FROM path_states WHERE path = $1"#, path.value)
            .execute(sqlite_transaction)
            .await;

        log::trace!(
            "DatahostStorageSQLite::delete_path_state; path: {:?}, query_result_r: {:?}",
            path,
            query_result_r
        );

        match query_result_r {
            Ok(query_result) => {
                if query_result.rows_affected() == 0 {
                    Err(DatahostStorageError::PathNotFound(path.clone()))
                } else {
                    Ok(())
                }
            }
            // NOTE: Not sure if this actually happens, or if it's always handled by above.
            Err(sqlx::Error::RowNotFound) => Err(DatahostStorageError::PathNotFound(path.clone())),
            Err(e) => Err(e.into()),
        }
    }
}
