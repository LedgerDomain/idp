# `rs/idp`

Rust SDK for Indoor Data Plumbing

## Rust Crate Contents

-   `idp` -- Top-level module for crate, which imports the `idp_*` crates as submodules.
    -   [`idp_client`](idp_client) aka `idp::client` -- Indoor Data Plumbing client, using GRPC.
    -   [`idp_core`](idp_core) aka `idp::core` -- Indoor Data Plumbing core data model and DB frontend.
    -   [`idp_proto`](idp_proto) aka `idp::proto` -- Indoor Data Plumbing protobufs and GRPC functionality.
    -   [`idp_server`](idp_server) aka `idp::server` -- Indoor Data Plumbing server, using GRPC.

## To-dos

-   Maybe use https://philcalcado.com/2018/11/19/a_structured_rfc_process.html within this git repo
    to document and moderate feature development process.
-   Using features to enable client and/or server code generation within `idp_proto` and generally in `idp`.
    This way no redundant/unused code is generated.
-   Each data type has an optional query method which can be used to query data out of that Plum.
-   Each data type has an optional associations method which can be used to determine which Plums
    a given Plum is related to.  Because some Plum bodies will be opaque data in certain contexts
    (e.g. sitting on server that is simply a datastore for clients), the Plum association data could/should
    potentially be a separate part of the Plum, so that it can be packaged and sent to authorized
    clients so that they can do the tracking themselves.  For example, an IDP server is running in
    "lights out" mode, meaning that it's not tracking the associations itself, but will send the
    association data to the client, which can track the associations and follow dependencies itself.
    Obviously this is slower than if the server can pre-compute associations on behalf of the client,
    but there are tradeoffs for "lights out".
-   The unit of data is the Plum, but higher order data structures (relational DBs, file hierarchies, etc)
    can be layered on top of the Plum layer.  This will be a conventional affordance to make IDP immediately
    understandable and usable.
-   Attempt to map the Plum Head/Body retrieval, as well as queries, onto HTTP/REST.  The fragment would
    be what provides the query, but the semantics of HTTP are that fragments are processed on the client
    side, which is not what IDP necessarily calls for.
-   Why is this different from/better than existing HTTP/REST patterns?  Because the basic unit of data,
    the Plum (Head and Body) provides:
    -   Data authentication (via authenticable hashes)
    -   Unique addressing of content
    Tracking associations between Plums provides:
    -   Automatic fetching of dependencies, so that content can be automatically and completely replicated
        across peers.
    Layering on specific data types, such as data types that contain Plum references, provide:
    -   DLT functionality (not just blockchain, but rather Merkle tree)
    -   Formal versioning (similar to git), which can even apply to DB states by tracking deltas to the DB.
-   How would the tracking of a relational DB work?
    -   State of the DB would be tracked formally via Plums that index:
        -   Previous state
-   How might this pattern be better than a traditional client/server pattern (even if a server is being used
    as the authoritative data source)?  You should use `git` as a starting point for your mental model of this.
    -   Client development could proceed without server involvement, because it's operating directly on
        Plum data and derived relational DBs (e.g. sqlite); what would otherwise be Plum-fetches from the server
        would just use synthetic/mock Plums.
    -   Client program itself could operate offline, because it fetches data to its local datastore, and then
        operates on it.
    -   Potentially the client program could handle almost all of the business logic on the app-specific data
        locally, and the server would just mediate interaction between users.
    -   This would unify the data models between client and server, because they're literally operating on the
        same data (Plums, as well as relational DBs and file hierarchies; relational DB part is a little handwavey
        because a server might use Postgres whereas a client might use SQLite).
    -   Server endpoints can be drastically simplified regarding data serialization, because client would just
        push its Plums to the server (and all their dependencies), and then in the client's traditional server
        requests, it would just use Plum references instead of actually uploading the data in the request.

        To be fair, this concept arguably exists in HTTP/REST already, say where a POST is how you upload data
        and the server returns a resource ID, and then you use that ID in later requests.  Though this pattern
        still requires the server and client to write all that data modeling, endpoint, and serialization by
        hand.

    -   A client/server model isn't even necessary, this can operate in a peer-to-peer context as well.
