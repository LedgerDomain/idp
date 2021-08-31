#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

// Not public, because we want to hide the DB details from the user of the wallet SDK,
// and only make pub exports of specific symbols.
mod data_host;
mod models;
mod schema;

pub use data_host::DataHost;
