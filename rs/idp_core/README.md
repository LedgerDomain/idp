# idp_core

Indoor Data Plumbing core data model and DB frontend.

## Running Tests

The tests run against a local SQLite database named `idp_datahost.rs`.  For full debug spew, run:

    RUST_LOG=trace cargo test -- --nocapture

## How to Define and Use Migrations

All the usual instructions at [getting started guide](https://diesel.rs/guides/getting-started)
apply, except for the need to specify the database URL, which for `idp_datahost` is defined using
the `IDP_DATAHOST_DATABASE_URL` env var instead of the usual `DATABASE_URL`.  The `diesel-migration`
shell script is meant to ease that, so instead of using `diesel migration ...`, use `diesel-migration ...`
as in the following examples:
-   Create a migration:

        ./diesel-migration generate my_fancy_migration_name

-   Run all pending migrations:

        ./diesel-migration run

-   Reverts and re-runs the most recently run migration.  Running this after creating and running a new migration
    is a good idea in order to test the `down.sql` part of the migration:

        ./diesel-migration redo

-   Reverts the most recently run migration:

        ./diesel-migration revert

Generally, see `diesel migration --help` (or `./diesel-migration --help`) for more.

## To-dos

-   Switch to `sqlx` for DB backends -- this is because it's simpler, cleaner, and supports async.
    In order to support multiple DB backends, a "DatahostStorage" trait should be defined which
    defines all the storage operations, and each DB backend has an implementation (which unfortunately
    has to be in its own crate in order to respect sqlx's compile-time SQL checking).
-   See about automating setting of "updated_at" timestamps in DB operations.
-   Consider using the `parking_lot` crate, as it apparently has sync primitives (Mutex, RwLock, etc)
    that are "smaller, faster, and more flexible than those in the Rust standard library".
-   Use protobufs only as a serialization format, not as the in-memory/API format.  Create API structs
    which are independent of any given serialization format.  This way, multiple different formats
    can be used for serialization.  This is partially motivated by the fact that `prost::Message`
    implements `Debug`, and one can't override its implementation (e.g. for Sha256Sum, which would
    ideally be printed as a hex string instead of as a byte array of decimal values).

## To-don'ts (I.e. Done)

-   Use [`diesel_migrations`](https://crates.io/crates/diesel_migrations) crate to run un-applied
    migrations at runtime, so that migrations don't have to be run as a separate process.
-   Create `PlumRef<T>` which uses a `PlumHeadSeal` to address a specific value, and which loads,
    deserializes, and caches the value into memory, making for a very powerful abstraction.
