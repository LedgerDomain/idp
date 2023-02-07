use crate::{
    models::{
        PlumBodyRow, PlumBodyRowInsertion, PlumHeadRow, PlumHeadRowInsertion, PlumRelationsRow,
        PlumRelationsRowInsertion,
    },
    BranchNode, DirNode, FragmentQueryResult, FragmentQueryable,
};
use anyhow::{Context, Result};
use diesel::{
    Connection,
    ExpressionMethods,
    QueryDsl,
    RunQueryDsl,
    SqliteConnection,
    // TextExpressionMethods,
};
use idp_proto::{
    Plum, PlumBody, PlumBodySeal, PlumHead, PlumHeadSeal, PlumRelations, RelationFlags,
};
use std::{collections::HashMap, convert::TryFrom};
// use uuid::Uuid;

// This makes it possible to run embedded_migrations::run(&conn) to apply migrations at runtime.
diesel_migrations::embed_migrations!("migrations");

pub struct Datahost {
    conn: SqliteConnection,
}

// TEMP HACK
unsafe impl Sync for Datahost {}

impl Datahost {
    /// This opens the datahost using an in-memory database.
    pub fn open_in_memory() -> Result<Self> {
        Self::open_database_url(":memory:")
    }

    /// This opens the datahost using the database_url specified by the IDP_DATAHOST_DATABASE_URL env var.
    pub fn open_using_env_var() -> Result<Self> {
        // Note that the .ok() call converts from Result<T,E> to Option<T>, producing None upon error.
        // This effectively ignores errors.
        dotenv::dotenv().ok();
        let database_url = std::env::var("IDP_DATAHOST_DATABASE_URL")
            .context("IDP_DATAHOST_DATABASE_URL env var must be set")?;
        log::info!(
            "Datahost database_url is being determined by IDP_DATAHOST_DATABASE_URL env var"
        );
        Self::open_database_url(&database_url)
    }

    /// This opens the datahost using the specified database_url.
    pub fn open_database_url(database_url: &str) -> Result<Self> {
        let conn = SqliteConnection::establish(&database_url).context(format!(
            "Error connecting to SQLite DB with database_url: {:#?}",
            database_url
        ))?;
        log::info!("Datahost opened using database_url: {:#?}", database_url);
        let datahost = Datahost { conn };
        datahost.run_migrations()?;
        Ok(datahost)
    }

    /// This consumes the Datahost instance and closes the DB connection.
    pub fn close(self) {
        // Nothing to do -- self will be dropped at the end of this method.
    }

    //
    // Data methods
    //

    pub fn store_plum_head(&self, plum_head: &PlumHead) -> Result<PlumHeadSeal> {
        log::trace!("Datahost::store_plum_head({:?})", plum_head);
        let plum_head_row_insertion = PlumHeadRowInsertion::from(plum_head);

        // Ideally we'd just use .on_conflict_do_nothing, but that method seems to be missing for some reason.
        match diesel::insert_into(crate::schema::plum_heads::table)
            .values(&plum_head_row_insertion)
            .execute(&self.conn)
        {
            Ok(_) => {
                // The PlumHead doesn't yet exist, but was successfully added.
                log::trace!("    success: stored {}", plum_head_row_insertion.head_seal);
            }
            Err(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            )) => {
                // The PlumHead already exists, so there's nothing to do.
                log::trace!(
                    "    success: already exists: {}",
                    plum_head_row_insertion.head_seal
                );
                // TODO: Query the DB and verify that the pushed PlumHead is identical to the existing one.
            }
            Err(e) => {
                // Note that this returns here because of the question mark being applied to Err.
                Err(e).context("Error inserting into plum_heads")?;
            }
        }

        Ok(plum_head_row_insertion.head_seal)
    }

    pub fn store_plum_relations(
        &self,
        source_head_seal: &PlumHeadSeal,
        plum_relations: &PlumRelations,
    ) -> Result<()> {
        log::trace!("Datahost::store_plum_relations({:?})", plum_relations);

        for relation_flags_mapping in &plum_relations.relation_flags_mappings {
            let plum_relations_row_insertion = PlumRelationsRowInsertion {
                source_head_seal: source_head_seal.clone(),
                target_head_seal: relation_flags_mapping.target_head_seal.clone(),
                relation_flags: RelationFlags::try_from(relation_flags_mapping.relation_flags_raw).expect("invalid RelationFlags value; if this panicking is a problem, then some 'invalid' or 'unknown' enum variant should be added to RelationFlags"),
            };
            log::debug!("inserting {:#?}", plum_relations_row_insertion);
            diesel::insert_into(crate::schema::plum_relations::table)
                .values(&plum_relations_row_insertion)
                .execute(&self.conn)?;
        }
        Ok(())
    }

    pub fn store_plum_body(&self, plum_body: &PlumBody) -> Result<PlumBodySeal> {
        log::trace!("Datahost::store_plum_body({:?})", plum_body);
        let plum_body_row_insertion = PlumBodyRowInsertion::from(plum_body);

        // Ideally we'd just use .on_conflict_do_nothing, but that method seems to be missing for some reason.
        match diesel::insert_into(crate::schema::plum_bodies::table)
            .values(&plum_body_row_insertion)
            .execute(&self.conn)
        {
            Ok(_) => {
                // Success, nothing to do.
                log::trace!("    success: stored {}", plum_body_row_insertion.body_seal);
            }
            Err(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            )) => {
                // The PlumBody already exists, so now update it.
                log::trace!(
                    "    PlumBody already exists: {}",
                    plum_body_row_insertion.body_seal
                );
                // TODO: Query the DB and verify that the pushed PlumBody is identical to the existing one.
            }
            Err(e) => {
                Err(e).context("Error inserting into plum_bodies")?;
            }
        }

        Ok(plum_body_row_insertion.body_seal)
    }

    pub fn store_plum(&self, plum: &Plum) -> Result<PlumHeadSeal> {
        self.store_plum_body(&plum.body)?;
        let plum_head_seal = self.store_plum_head(&plum.head)?;
        if let Some(relations) = &plum.relations_o {
            self.store_plum_relations(&plum_head_seal, relations)?;
        }
        Ok(plum_head_seal)
    }

    pub fn load_plum_head(&self, plum_head_seal: &PlumHeadSeal) -> Result<PlumHead> {
        Ok(self.select_plum_head_row(plum_head_seal)?.into())
    }

    //     pub fn load_plum_relations(&self, plum_head_seal: &PlumHeadSeal) -> Result<PlumRelations> {
    //         TODO -- implement
    //     }

    pub fn load_plum_body(&self, plum_body_seal: &PlumBodySeal) -> Result<PlumBody> {
        use std::convert::TryInto;
        Ok(self.select_plum_body_row(plum_body_seal)?.try_into()?)
    }

    pub fn load_plum(&self, plum_head_seal: &PlumHeadSeal) -> Result<Plum> {
        let head = self.load_plum_head(plum_head_seal)?;
        let relations_o = match &head.relations_seal_o {
            //             Some(relations_seal) => self.load_plum_relations(relations_seal),
            //             None => None,
            _ => None, // TEMP HACK
        };
        let body = self.load_plum_body(&head.body_seal)?;
        Ok(Plum {
            head,
            relations_o,
            body,
        })
    }

    //     pub fn delete_plum_head(&self, plum_head_seal: &PlumHeadSeal) -> Result<()> {
    //         self.conn.transaction(|| {
    //             {
    //                 use crate::schema::plum_heads::dsl;
    //
    //
    //                 diesel::delete(
    //                     crate::schema::plum_heads::table.filter(dsl::head_seal.eq(plum_head_seal))
    //                 ).execute(&self.conn)?;
    //             }
    //
    //
    //         })?
    //         Ok(())
    //     }
    //
    //     /// Note that this forces deletion of the PlumBody, regardless of if its reference_count is positive.
    //     pub fn delete_plum_body(&self, plum_body_seal: &PlumBodySeal) -> Result<()> {
    //         self.conn.transaction(|| {
    //         })?
    //         use crate::schema::plum_bodies::dsl;
    //         diesel::delete(
    //             crate::schema::plum_bodies::table.filter(dsl::body_seal.eq(plum_body_seal))
    //         ).execute(&self.conn)?;
    //         Ok(())
    //     }
    //
    //     pub fn delete_plum(&self, plum_head_seal: &PlumHeadSeal) -> Result<()> {
    //         self.delete_plum_head(plum_head_seal)
    //         let head = self.delete_plum_head(plum_head_seal)?;
    //         let body = self.delete_plum_body(&head.body_seal)?;
    //         Ok(Plum { head, body })
    //     }

    fn select_plum_head_row(&self, plum_head_seal: &PlumHeadSeal) -> Result<PlumHeadRow> {
        use crate::schema::plum_heads::dsl;
        Ok(dsl::plum_heads
            .filter(dsl::head_seal.eq(plum_head_seal))
            .limit(1)
            // This should return a Vec<PlumHeadRow> with exactly 0 or 1 element(s)
            .load::<PlumHeadRow>(&self.conn)
            .context("Error loading plum_heads")?
            // This should return Some(plum_head_row)
            // TODO: Use first here instead?
            .pop()
            .ok_or_else(|| anyhow::format_err!("PlumHeadSeal {} not found", plum_head_seal))?)
    }

    fn select_plum_body_row(&self, plum_body_seal: &PlumBodySeal) -> Result<PlumBodyRow> {
        use crate::schema::plum_bodies::dsl;
        Ok(dsl::plum_bodies
            .filter(dsl::body_seal.eq(plum_body_seal))
            .limit(1)
            // This should return a Vec<PlumBodyRow> with exactly 0 or 1 element(s)
            .load::<PlumBodyRow>(&self.conn)
            .context("Error loading plum_bodies")?
            // This should return Some(plum_body_row)
            // TODO: Use first here instead?
            .pop()
            .ok_or_else(|| anyhow::format_err!("PlumBodySeal {} not found", plum_body_seal))?)
    }

    // TEMP HACK -- this should be private
    pub fn select_plum_body_reference_count(&self, plum_body_seal: &PlumBodySeal) -> Result<i64> {
        use crate::schema::plum_heads::dsl;
        Ok(dsl::plum_heads
            .select(diesel::dsl::count_star())
            .filter(dsl::body_seal.eq(plum_body_seal))
            .first(&self.conn)
            .context("Error counting plum_body references")?)
    }

    //
    // Methods for determining relations between Plums
    //

    pub fn accumulated_relations_recursive(
        &self,
        plum_head_seal: &PlumHeadSeal,
        mask: RelationFlags,
    ) -> Result<HashMap<PlumHeadSeal, RelationFlags>> {
        let mut relation_flags_m = HashMap::new();
        self.accumulate_relations_recursive_impl(plum_head_seal, mask, &mut relation_flags_m)?;
        Ok(relation_flags_m)
    }

    fn accumulate_relations_recursive_impl(
        &self,
        plum_head_seal: &PlumHeadSeal,
        mask: RelationFlags,
        relation_flags_m: &mut HashMap<PlumHeadSeal, RelationFlags>,
        // TODO: Need some way of indicating which Plums didn't have relations present in the Datahost,
        // so that the client can act appropriately.
    ) -> Result<()> {
        // This implementation is probably horribly wrong for if/when there are Relation cycles.

        if relation_flags_m.contains_key(plum_head_seal) {
            // Already traversed; nothing to do.
            return Ok(());
        }

        // Recurse on the relations for this plum_head_seal.
        let inner_relation_flags_m = {
            let mut inner_relation_flags_m: HashMap<PlumHeadSeal, RelationFlags> = HashMap::new();
            use crate::schema::plum_relations::dsl;
            let plum_relations_row_v = dsl::plum_relations
                .filter(dsl::source_head_seal.eq(plum_head_seal))
                // This should return a Vec<PlumRelationsRow>
                .load::<PlumRelationsRow>(&self.conn)
                .context("Error loading plum_heads")?;
            for plum_relations_row in plum_relations_row_v {
                log::trace!(
                    "accumulate_relations_recursive_impl; {} -> {}",
                    plum_relations_row.source_head_seal,
                    plum_relations_row.target_head_seal
                );
                let masked_relation_flags = mask & plum_relations_row.relation_flags;
                // Only do anything if the masked flags are nonzero.
                if masked_relation_flags != RelationFlags::NONE {
                    match inner_relation_flags_m.get_mut(&plum_relations_row.target_head_seal) {
                        Some(inner_relation_flags) => {
                            *inner_relation_flags |= masked_relation_flags;
                        }
                        None => {
                            inner_relation_flags_m.insert(
                                plum_relations_row.target_head_seal.clone(),
                                masked_relation_flags,
                            );
                        }
                    }
                }
            }
            inner_relation_flags_m
        };

        // Now go through the accumulated inner_relation_flags_m and recurse.
        for (inner_plum_head_seal, inner_relation_flags) in inner_relation_flags_m.iter() {
            // Just make sure that inner_relation_flags obeys the mask constraint.
            assert_eq!(*inner_relation_flags & !mask, RelationFlags::NONE);

            // NOTE that we're passing mask here, instead of inner_relation_flags, meaning that the
            // full mask will "bypass" any RelationFlag "bottleneck" imposed by a particular data type.
            // For example, only CONTENT_DEPENDENCY is used by DirNode, but if mask includes
            // METADATA_DEPENDENCY, then on querying a child of the DirNode for its relations,
            // METADATA_DEPENDENCY will be fair game again.  This may or may not be what is actually
            // desired.  Will determine through testing.
            log::trace!(
                "accumulate_relations_recursive_impl; recursing on {}",
                inner_plum_head_seal
            );
            self.accumulate_relations_recursive_impl(
                inner_plum_head_seal,
                mask.clone(),
                relation_flags_m,
            )?;

            // Add inner_plum_head_seal with its computed inner_relation_flags to mark as traversed.
            log::trace!(
                "accumulate_relations_recursive_impl; adding to relation_flags_m: {} -> {:?}",
                inner_plum_head_seal,
                inner_relation_flags
            );
            relation_flags_m.insert(inner_plum_head_seal.clone(), *inner_relation_flags);
        }

        Ok(())
    }

    //
    // Methods for fragment query
    //

    // TODO: Eventually make this return Box<Any> or something
    pub fn fragment_query(
        &self,
        starting_plum_head_seal: &PlumHeadSeal,
        query_str: &str,
    ) -> Result<PlumHeadSeal> {
        let mut current_plum_head_seal = starting_plum_head_seal.clone();
        let mut current_query_str = query_str;
        loop {
            let plum_head_row = self.select_plum_head_row(&current_plum_head_seal)?;
            let fragment_query_result =
                match std::str::from_utf8(plum_head_row.body_content_type.as_ref()) {
                    // TODO: Replace this with a callback registry pattern
                    Ok("idp::BranchNode") => {
                        log::trace!("fragment_query; deserializing idp::BranchNode");
                        let plum_body_row = self.select_plum_body_row(&plum_head_row.body_seal)?;
                        if plum_body_row.body_content_o.is_none() {
                            return Err(anyhow::format_err!(
                                "Plum {} had missing body_content",
                                current_plum_head_seal
                            ));
                        }
                        // Deserialize body_content and call fragment_query_single_segment.
                        let body_content = plum_body_row.body_content_o.unwrap();
                        let branch_node: BranchNode = rmp_serde::from_read_ref(&body_content)?;
                        branch_node.fragment_query_single_segment(
                            &current_plum_head_seal,
                            current_query_str,
                        )?
                    }
                    Ok("idp::DirNode") => {
                        log::trace!("fragment_query; deserializing idp::BranchNode");
                        let plum_body_row = self.select_plum_body_row(&plum_head_row.body_seal)?;
                        if plum_body_row.body_content_o.is_none() {
                            return Err(anyhow::format_err!(
                                "Plum {} had missing body_content",
                                current_plum_head_seal
                            ));
                        }
                        // Deserialize body_content and call fragment_query_single_segment.
                        let body_content = plum_body_row.body_content_o.unwrap();
                        let dir_node: DirNode = rmp_serde::from_read_ref(&body_content)?;
                        dir_node.fragment_query_single_segment(
                            &current_plum_head_seal,
                            current_query_str,
                        )?
                    }
                    _ => {
                        // This data type is considered FragmentQueryable-opaque, so produce an error.
                        // Later, this should just return the body_content.  But for now, for simplicity,
                        // the fragment query returns PlumHeadSeal.
                        return Err(anyhow::format_err!(
                        "not yet supported; This data type is considered FragmentQueryable-opaque"
                    ));
                    }
                };
            match fragment_query_result {
                FragmentQueryResult::Value(plum_head_seal) => {
                    // We reached the end of the query, so return.
                    return Ok(plum_head_seal);
                }
                FragmentQueryResult::ForwardQueryTo {
                    target,
                    rest_of_query_str,
                } => {
                    // The query must continue.
                    // This assert is to ensure the finite-time termination of this loop.
                    assert!(rest_of_query_str.len() < current_query_str.len());
                    // Update the "current" vars for the next iteration.
                    current_plum_head_seal = target;
                    current_query_str = rest_of_query_str;
                }
            }
        }
    }

    //
    // Private implementation methods
    //

    fn run_migrations(&self) -> Result<()> {
        embedded_migrations::run(&self.conn)?;
        log::debug!("Migrations successfully run on Datahost DB");
        Ok(())
    }
}

impl Drop for Datahost {
    fn drop(&mut self) {
        log::info!("Datahost closed");
    }
}
