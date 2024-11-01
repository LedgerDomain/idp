# Rust workspace for Indoor Data Plumbing

See [a slide deck describing the Indoor Data Plumbing concept](https://docs.google.com/presentation/d/16azx22pCv_JvRslsRaSUPUSqTLQjyJ8oQZ-7-J3t1gw/edit?usp=sharing).

## Contents

-   `examples` -- Example programs using Indoor Data Plumbing.
    -   `pl` -- A basic, interpreted programming language which demonstrates use of `PlumRef`.
-   `idp` -- Crate for Indoor Data Plumbing SDK (Rust).
-   `idp_core` -- Crate for Indoor Data Plumbing core and client SDK (Rust).
-   `idp_datahost_storage_sqlite` -- Implementation of the `DatahostStorage` trait providing a SQLite DB for `Datahost` persistence.
-   `idp_proto` -- Crate for Indoor Data Plumbing proto SDK (Rust)
-   `idp_server` -- Crate for Indoor Data Plumbing server SDK (Rust)
