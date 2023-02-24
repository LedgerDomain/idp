# idp_core

Indoor Data Plumbing core data model and DB frontend.

## Running Tests

The tests run against a local SQLite database named `idp_core_tests.db`.  For full debug spew, run:

    RUST_BACKTRACE=1 RUST_LOG=trace cargo test --package idp_core --all-features -- --nocapture

## To-dos

-   Probably create an idp API consisting of all the datatypes currently in idp_proto (but not the
    GRPC-specific ones), and then rename `idp_proto` to `idp_grpc` and make that all GRPC-specific
    stuff.  The idea here being that multiple different protocols can be used to implement IDP nodes,
    such as GRPC, libp2p, JSON-RPC, HTTP REST, etc.
    -   Use protobufs only as a serialization format, not as the in-memory/API format.  And perhaps only
        use protobufs for GRPC services.  Create API structs which are independent of any given serialization
        format.  This way, multiple different formats can be used for serialization.  This is partially
        motivated by the fact that `prost::Message` implements `Debug`, and one can't override its
        implementation (e.g. for Sha256Sum, which would ideally be printed as a hex string instead of
        as a byte array of decimal values).
-   Make sure that the seal is checked on all sealed values upon transfer between Datahosts.
-   Add an "has_broken_relations" boolean to the plum_heads table, so that efficient dependency breakage can
    be tracked.  Perhaps track the number of expected relations and the number of stored relations.
-   Add "previous version" relation, which would be used e.g. in `BranchNode`, since when fetching a piece
    of data that's linked into a data structure via `PlumRef`, one wouldn't want the entire history of
    that data.
-   Add non-DAG kinds of `PlumRelation`s where there can be cycles.  For example, "hyperlink to"
    could have a cycle in which two documents refer to one another.  Though actually this is not
    directly possible, because relations are addressed via `PlumHeadSeal`, and those can't be known
    in advance, so even forming a cycle of relations is infeasible.  It would require one level
    of indirection, such as a piece of mutable state being addressed via (e.g.) a URL.
-   Efficient implementations of push and pull; use GRPC streaming to handle multiple requests within
    the same connection.  Though if there's some sort of keepalive, then streaming may not matter.
    -   Dumb implementations would simply assume that dependency trees can't be incomplete
        from below (meaning if a Plum is present all its dependencies are present).
    -   Correct implementations would do dependency completeness tracking.

## To-don'ts (I.e. Done)

-   Create `PlumRef<T>` which uses a `PlumHeadSeal` to address a specific value, and which loads,
    deserializes, and caches the value into memory, making for a very powerful abstraction.
-   Use the `async-lock` crate, as its RwLock guards are *actually* `Send`, unlike `parking_lot`'s.
    Also it has upgradeable RwLock guards, which could be useful.
-   Switch to `sqlx` for DB backends -- this is because it's simpler, cleaner, and supports async.
    In order to support multiple DB backends, a "DatahostStorage" trait should be defined which
    defines all the storage operations, and each DB backend has an implementation (which unfortunately
    has to be in its own crate in order to respect sqlx's compile-time SQL checking).
-   Add `PlumRef` capability to retrieve from remote `Datahost`.  This will need a notion of an address
    of a remote datahost.  That should probably look like a URI.  It could be e.g.

        idp://<hostname>[:port]/<plum-head-seal>

    The following URI might be used to indicate the plum head seal only, and not its origin:

        idp:///<plum-head-seal>

    And of course, the fragment query can be appended to a URI, e.g.

        idp://<hostname>[:port]/<plum-head-seal>#<fragment-query-string>

