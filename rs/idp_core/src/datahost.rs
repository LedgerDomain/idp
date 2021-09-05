use crate::{
    BranchNode,
    DirNode,
    FragmentQueryable,
    FragmentQueryResult,
    models::{
        PlumBodyRow,
        PlumBodyRowInsertion,
        PlumHeadRow,
        PlumHeadRowInsertion,
    },
    Relational,
    RelationFlags,
};
use diesel::{
    Connection,
    ExpressionMethods,
    QueryDsl,
    RunQueryDsl,
    SqliteConnection,
    // TextExpressionMethods,
};
use failure::ResultExt; // Needed for .context("...") on errors.
use idp_proto::{Plum, PlumBody, PlumBodySeal, PlumHead, PlumHeadSeal};
use std::collections::HashMap;
// use uuid::Uuid;

// This makes it possible to run embedded_migrations::run(&conn) to apply migrations at runtime.
diesel_migrations::embed_migrations!("migrations");

pub struct Datahost {
    conn: SqliteConnection,
}

impl Datahost {
    /// This opens the datahost using an in-memory database.
    pub fn open_in_memory() -> Result<Self, failure::Error> {
        Self::open_database_url(":memory:")
    }

    /// This opens the datahost using the database_url specified by the IDP_DATAHOST_DATABASE_URL env var.
    pub fn open_using_env_var() -> Result<Self, failure::Error> {
        // Note that the .ok() call converts from Result<T,E> to Option<T>, producing None upon error.
        // This effectively ignores errors.
        dotenv::dotenv().ok();
        let database_url = std::env::var("IDP_DATAHOST_DATABASE_URL")
            .context("IDP_DATAHOST_DATABASE_URL env var must be set")?;
        log::info!("Datahost database_url is being determined by IDP_DATAHOST_DATABASE_URL env var");
        Self::open_database_url(&database_url)
    }

    /// This opens the datahost using the specified database_url.
    pub fn open_database_url(database_url: &str) -> Result<Self, failure::Error> {
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

    pub fn create_plum_head(&self, plum_head: &PlumHead) -> Result<PlumHeadSeal, failure::Error> {
        log::trace!("Datahost::create_plum_head({:?})", plum_head);
        let plum_head_row_insertion = PlumHeadRowInsertion::from(plum_head);

        // Ideally we'd just use .on_conflict_do_nothing, but that method seems to be missing for some reason.
        match diesel::insert_into(crate::schema::plum_heads::table)
            .values(&plum_head_row_insertion)
            .execute(&self.conn)
        {
            Ok(_) => {
                // The PlumHead doesn't yet exist, but was successfully added.
                log::trace!("    success: created {}", plum_head_row_insertion.head_seal);
            }
            Err(diesel::result::Error::DatabaseError(diesel::result::DatabaseErrorKind::UniqueViolation, _)) => {
                // The PlumHead already exists, so there's nothing to do.
                log::trace!("    success: already exists: {}", plum_head_row_insertion.head_seal);
                // TODO: Query the DB and verify that the pushed PlumHead is identical to the existing one.
            }
            Err(e) => {
                // Note that this returns here because of the question mark being applied to Err.
                Err(e).context("Error inserting into plum_heads")?;
            }
        }

        Ok(plum_head_row_insertion.head_seal)
    }

    pub fn create_plum_body(&self, plum_body: &PlumBody) -> Result<PlumBodySeal, failure::Error> {
        log::trace!("Datahost::create_plum_body({:?})", plum_body);
        let plum_body_row_insertion = PlumBodyRowInsertion::from(plum_body);

        // Ideally we'd just use .on_conflict_do_nothing, but that method seems to be missing for some reason.
        match diesel::insert_into(crate::schema::plum_bodies::table)
            .values(&plum_body_row_insertion)
            .execute(&self.conn)
        {
            Ok(_) => {
                // Success, nothing to do.
                log::trace!("    success: created {}", plum_body_row_insertion.body_seal);
            }
            Err(diesel::result::Error::DatabaseError(diesel::result::DatabaseErrorKind::UniqueViolation, _)) => {
                // The PlumBody already exists, so now update it.
                log::trace!("    PlumBody already exists: {}", plum_body_row_insertion.body_seal);
                // TODO: Query the DB and verify that the pushed PlumBody is identical to the existing one.

            }
            Err(e) => {
                Err(e).context("Error inserting into plum_bodies")?;
            }
        }

        Ok(plum_body_row_insertion.body_seal)
    }

    pub fn create_plum(&self, plum: &Plum) -> Result<PlumHeadSeal, failure::Error> {
        self.create_plum_body(&plum.body)?;
        self.create_plum_head(&plum.head)
    }

    pub fn read_plum_head(&self, plum_head_seal: &PlumHeadSeal) -> Result<PlumHead, failure::Error> {
        Ok(self.select_plum_head_row(plum_head_seal)?.into())
    }

    pub fn read_plum_body(&self, plum_body_seal: &PlumBodySeal) -> Result<PlumBody, failure::Error> {
        use std::convert::TryInto;
        Ok(self.select_plum_body_row(plum_body_seal)?.try_into()?)
    }

    pub fn read_plum(&self, plum_head_seal: &PlumHeadSeal) -> Result<Plum, failure::Error> {
        let head = self.read_plum_head(plum_head_seal)?;
        let body = self.read_plum_body(&head.body_seal)?;
        Ok(Plum { head, body })
    }

//     pub fn delete_plum_head(&self, plum_head_seal: &PlumHeadSeal) -> Result<(), failure::Error> {
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
//     pub fn delete_plum_body(&self, plum_body_seal: &PlumBodySeal) -> Result<(), failure::Error> {
//         self.conn.transaction(|| {
//         })?
//         use crate::schema::plum_bodies::dsl;
//         diesel::delete(
//             crate::schema::plum_bodies::table.filter(dsl::body_seal.eq(plum_body_seal))
//         ).execute(&self.conn)?;
//         Ok(())
//     }
//
//     pub fn delete_plum(&self, plum_head_seal: &PlumHeadSeal) -> Result<(), failure::Error> {
//         self.delete_plum_head(plum_head_seal)
//         let head = self.delete_plum_head(plum_head_seal)?;
//         let body = self.delete_plum_body(&head.body_seal)?;
//         Ok(Plum { head, body })
//     }

    fn select_plum_head_row(&self, plum_head_seal: &PlumHeadSeal) -> Result<PlumHeadRow, failure::Error> {
        use crate::schema::plum_heads::dsl;
        Ok(dsl::plum_heads
            .filter(dsl::head_seal.eq(plum_head_seal))
            .limit(1)
            // This should return a Vec<PlumHeadRow> with exactly 0 or 1 element(s)
            .load::<PlumHeadRow>(&self.conn)
            .context("Error loading plum_heads")?
            // This should return Some(plum_head_row)
            .pop()
            .ok_or_else(|| failure::format_err!("PlumHeadSeal {} not found", plum_head_seal))?
        )
    }

    fn select_plum_body_row(&self, plum_body_seal: &PlumBodySeal) -> Result<PlumBodyRow, failure::Error> {
        use crate::schema::plum_bodies::dsl;
        Ok(dsl::plum_bodies
            .filter(dsl::body_seal.eq(plum_body_seal))
            .limit(1)
            // This should return a Vec<PlumBodyRow> with exactly 0 or 1 element(s)
            .load::<PlumBodyRow>(&self.conn)
            .context("Error loading plum_bodies")?
            // This should return Some(plum_body_row)
            .pop()
            .ok_or_else(|| failure::format_err!("PlumBodySeal {} not found", plum_body_seal))?
        )
    }

    // TEMP HACK -- this should be private
    pub fn select_plum_body_reference_count(&self, plum_body_seal: &PlumBodySeal) -> Result<i64, failure::Error> {
        use crate::schema::plum_heads::dsl;
        Ok(dsl::plum_heads
            .select(diesel::dsl::count_star())
            .filter(dsl::body_seal.eq(plum_body_seal))
            .first(&self.conn)
            .context("Error counting plum_body references")?
        )
    }

    //
    // Methods for determining relations between Plums
    //

    pub fn accumulated_relations_recursive(
        &self,
        plum_head_seal: &PlumHeadSeal,
        mask: RelationFlags,
    ) -> Result<HashMap<PlumHeadSeal, RelationFlags>, failure::Error> {
        let mut relation_m = HashMap::new();
        self.accumulate_relations_recursive_impl(plum_head_seal, mask, &mut relation_m)?;
        Ok(relation_m)
    }

    fn accumulate_relations_recursive_impl(
        &self,
        plum_head_seal: &PlumHeadSeal,
        mask: RelationFlags,
        relation_m: &mut HashMap<PlumHeadSeal, RelationFlags>,
    ) -> Result<(), failure::Error> {
        // This implementation is probably horribly wrong for if/when there are Relation cycles.

        if relation_m.contains_key(plum_head_seal) {
            // Already traversed; nothing to do.
            return Ok(())
        }

        let mut inner_relation_m: HashMap<PlumHeadSeal, RelationFlags> = HashMap::new();
        // TEMP HACK: For now just use some hardcoded values of body_content_type to determine if
        // a Relation query can be done on the PlumBody.
        let plum_head_row = self.select_plum_head_row(plum_head_seal)?;
        match std::str::from_utf8(plum_head_row.body_content_type.as_ref()) {
            Ok("idp::BranchNode") => {
                log::trace!("accumulate_relations_recursive_impl; deserializing idp::BranchNode");
                let plum_body_row = self.select_plum_body_row(&plum_head_row.body_seal)?;
                // If body_content_o is not None, then deserialize and accumulate relations.
                if let Some(body_content) = plum_body_row.body_content_o {
                    let branch_node: BranchNode = rmp_serde::from_read_ref(&body_content)?;
                    branch_node.accumulate_relations_nonrecursive(&mut inner_relation_m, mask.clone())?;
                }
            }
            Ok("idp::DirNode") => {
                log::trace!("accumulate_relations_recursive_impl; deserializing idp::DirNode");
                let plum_body_row = self.select_plum_body_row(&plum_head_row.body_seal)?;
                // If body_content_o is not None, then deserialize and accumulate relations.
                if let Some(body_content) = plum_body_row.body_content_o {
                    let dir_node: DirNode = rmp_serde::from_read_ref(&body_content)?;
                    dir_node.accumulate_relations_nonrecursive(&mut inner_relation_m, mask.clone())?;
                }
            }
            _ => {
                // This data type is considered Relation-opaque, so don't traverse.  But later,
                // some data types might implement Relational.  Or, more likely, Plums will have
                // a relations attribute which allows them to define their own relations metadata,
                // so this Datahost doesn't have to parse the body (because sometimes it won't
                // be able to).
            }
        }

        // Now go through the accumulated inner_relation_m and recurse.
        for (inner_plum_head_seal, inner_relation_flags) in inner_relation_m.iter() {
            // Just make sure that inner_relation_flags obeys the mask constraint.
            assert_eq!(*inner_relation_flags & !mask, RelationFlags::NONE);

            // NOTE that we're passing mask here, instead of inner_relation_flags, meaning that the
            // full mask will "bypass" any RelationFlag "bottleneck" imposed by a particular data type.
            // For example, only CONTENT_DEPENDENCY is used by DirNode, but if mask includes
            // METADATA_DEPENDENCY, then on querying a child of the DirNode for its relations,
            // METADATA_DEPENDENCY will be fair game again.  This may or may not be what is actually
            // desired.  Will determine through testing.
            log::trace!("accumulate_relations_recursive_impl; recursing on {}", inner_plum_head_seal);
            self.accumulate_relations_recursive_impl(inner_plum_head_seal, mask.clone(), relation_m)?;

            // Add inner_plum_head_seal with its computed inner_relation_flags to mark as traversed.
            log::trace!("accumulate_relations_recursive_impl; adding to relation_m: {} -> {:?}", inner_plum_head_seal, inner_relation_flags);
            relation_m.insert(inner_plum_head_seal.clone(), *inner_relation_flags);
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
    ) -> Result<PlumHeadSeal, failure::Error> {
        let mut current_plum_head_seal = starting_plum_head_seal.clone();
        let mut current_query_str = query_str;
        loop {
            let plum_head_row = self.select_plum_head_row(&current_plum_head_seal)?;
            let fragment_query_result = match std::str::from_utf8(plum_head_row.body_content_type.as_ref()) {
                Ok("idp::BranchNode") => {
                    log::trace!("fragment_query; deserializing idp::BranchNode");
                    let plum_body_row = self.select_plum_body_row(&plum_head_row.body_seal)?;
                    if plum_body_row.body_content_o.is_none() {
                        return Err(failure::format_err!("Plum {} had missing body_content", current_plum_head_seal));
                    }
                    // Deserialize body_content and call fragment_query_single_segment.
                    let body_content = plum_body_row.body_content_o.unwrap();
                    let branch_node: BranchNode = rmp_serde::from_read_ref(&body_content)?;
                    branch_node.fragment_query_single_segment(&current_plum_head_seal, current_query_str)?
                }
                Ok("idp::DirNode") => {
                    log::trace!("fragment_query; deserializing idp::BranchNode");
                    let plum_body_row = self.select_plum_body_row(&plum_head_row.body_seal)?;
                    if plum_body_row.body_content_o.is_none() {
                        return Err(failure::format_err!("Plum {} had missing body_content", current_plum_head_seal));
                    }
                    // Deserialize body_content and call fragment_query_single_segment.
                    let body_content = plum_body_row.body_content_o.unwrap();
                    let branch_node: BranchNode = rmp_serde::from_read_ref(&body_content)?;
                    branch_node.fragment_query_single_segment(&current_plum_head_seal, current_query_str)?
                }
                _ => {
                    // This data type is considered FragmentQueryable-opaque, so produce an error.
                    // Later, this should just return the body_content.  But for now, for simplicity,
                    // the fragment query returns PlumHeadSeal.
                    return Err(failure::format_err!("not yet supported; This data type is considered FragmentQueryable-opaque"));
                }
            };
            match fragment_query_result {
                FragmentQueryResult::Value(plum_head_seal) => {
                    // We reached the end of the query, so return.
                    return Ok(plum_head_seal);
                }
                FragmentQueryResult::ForwardQueryTo { target, rest_of_query_str } => {
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

    fn run_migrations(&self) -> Result<(), failure::Error> {
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
