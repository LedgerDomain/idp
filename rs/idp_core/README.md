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

-   Use https://github.com/adwhit/diesel-derive-enum for enums in tables.
-   Use separate DB backends, using the following structures:
    -   Use a subdir for each backend, vis a vis migrations and `.env` file
        -   `mysql`
            -   `.env` -- defines the IDP_DATABASE_URL env var.
            -   `diesel.toml` -- configuration for diesel CLI tool (including usage of `diesel-migration`)
            -   `diesel-migration` -- script that facilitates management of migrations.
            -   `migrations` -- dir that hosts the migrations SQL.
        -   `postgres`
            -   `.env`
            -   `diesel.toml`
            -   `diesel-migration`
            -   `migrations`
        -   `sqlite`
            -   `.env`
            -   `diesel.toml`
            -   `diesel-migration`
            -   `migrations`
    -   Point the `schema.rs` file at subdirs within the `src` dir.
        -   `src`
            -   `mysql`
                -   `schema.rs`
            -   `postgres`
                -   `schema.rs`
            -   `sqlite`
                -   `schema.rs`
-   Use https://github.com/diesel-rs/diesel/blob/master/diesel/src/sqlite/connection/diesel_manage_updated_at.sql
    to automatically handle `updated_at`.  Also see https://diesel.rs/guides/all-about-inserts.html

## To-don'ts (I.e. Done)

-   Use [`diesel_migrations`](https://crates.io/crates/diesel_migrations) crate to run un-applied
    migrations at runtime, so that migrations don't have to be run as a separate process.
