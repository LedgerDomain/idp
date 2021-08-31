use crate::{
    models::{
        PlumBodyRow,
        PlumBodyRowInsertion,
        PlumHeadRow,
        PlumHeadRowInsertion,
    },
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
        log::debug!("Datahost database_url is being determined by IDP_DATAHOST_DATABASE_URL env var");
        Self::open_database_url(&database_url)
    }

    /// This opens the datahost using the specified database_url.
    pub fn open_database_url(database_url: &str) -> Result<Self, failure::Error> {
        let conn = SqliteConnection::establish(&database_url).context(format!(
            "Error connecting to SQLite DB with database_url: {:#?}",
            database_url
        ))?;
        log::debug!("Datahost opened using database_url: {:#?}", database_url);
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
        log::debug!("Datahost::create_plum_head({:?})", plum_head);
        let plum_head_row_insertion = PlumHeadRowInsertion::from(plum_head);

        // Ideally we'd just use .on_conflict_do_nothing, but that method seems to be missing for some reason.
        match diesel::insert_into(crate::schema::plum_heads::table)
            .values(&plum_head_row_insertion)
            .execute(&self.conn)
        {
            Ok(_) => {
                // The PlumHead doesn't yet exist, but was successfully added.
                log::debug!("    success: pushed {:?}", plum_head_row_insertion.head_seal);
            }
            Err(diesel::result::Error::DatabaseError(diesel::result::DatabaseErrorKind::UniqueViolation, _)) => {
                // The PlumHead already exists, so there's nothing to do.
                log::debug!("    success: already exists: {:?}", plum_head_row_insertion.head_seal);
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
        log::debug!("Datahost::create_plum_body({:?})", plum_body);
        let plum_body_row_insertion = PlumBodyRowInsertion::from(plum_body);

        // Ideally we'd just use .on_conflict_do_nothing, but that method seems to be missing for some reason.
        match diesel::insert_into(crate::schema::plum_bodies::table)
            .values(&plum_body_row_insertion)
            .execute(&self.conn)
        {
            Ok(_) => {
                // Success, nothing to do.
                log::debug!("    success: pushed {:?}", plum_body_row_insertion.body_seal);
            }
            Err(diesel::result::Error::DatabaseError(diesel::result::DatabaseErrorKind::UniqueViolation, _)) => {
                // The PlumBody already exists, so now update it.
                log::debug!("    PlumBody already exists: {:?}", plum_body_row_insertion.body_seal);
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
            .unwrap()
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
            .unwrap()
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

    fn accumulate_relations_recursive(
        &self,
        plum_head_seal: &PlumHeadSeal,
        mask: RelationFlags,
    ) -> Result<(), failure::Error> {
        let relation_m: mut HashMap<PlumHeadSeal, RelationFlags> = HashMap::new();
        Ok(self.accumulate_relations_recursive_impl(plum_head_seal, mask, relation_m)?)
    }

    fn accumulate_relations_recursive_impl(
        &self,
        plum_head_seal: &PlumHeadSeal,
        mask: RelationFlags,
        relation_m: &mut HashMap<PlumHeadSeal, RelationFlags>,
    ) -> Result<(), failure::Error> {
        if relation_m.contains_key(plum_head_seal) {
            // Already traversed; nothing to do.
            return Ok(())
        }

        let inner_relation_m: mut HashMap<PlumHeadSeal, RelationFlags> = HashMap::new();
        // TODO: Need to be able to deserialize a Plum into std::any::Any (maybe as `Box<dyn Any>`?)
        // then check if that type implements Relational, and if so, call Relational::accumulate_relations
        // on the deserialized thing.
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
        log::debug!("Datahost closed");
    }
}
