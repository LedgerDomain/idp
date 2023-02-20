# idp_datahost_storage_sqlite

SQLite-backed implementation of `DatahostStorage` trait using the `sqlx` Rust crate.

## How to Define and Use DB Migrations

First, ensure that the `sqlx` CLI tool is installed.  The following installs `sqlx` CLI tool for all databases supported by `sqlx`.

    cargo install sqlx-cli

Reference: Instructions: https://crates.io/crates/sqlx-cli

Now, ensure that the migrations DB (which is also the DB against which `sqlx` checks SQL statements at compile time) is set up.

    sqlx database setup

This should create a DB called `idp_datahost_storage_sqlite_migrations.db` and run all existing migrations against it.

In order to create a new migration, run

    sqlx migrate add -r <migration-name>

This will create a migration with the given name with both "up" and "down" scripts.  The "up" script should apply the migration, and the "down" script should reverse the actions in "up".
