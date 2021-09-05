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
    -   The server (when operating in server/client mode) effectively is a superuser, because it's
        privy to all the data.  It could do traversals of association data for the client subject
        to the client's permissions (e.g. determining the full tree of dependencies for a given Plum).
-   The unit of data is the Plum, but higher order data structures (relational DBs, file hierarchies, etc)
    can be layered on top of the Plum layer.  This will be a conventional affordance to make IDP immediately
    understandable and usable.
-   Attempt to map the Plum Head/Body retrieval, as well as queries, onto URLs with HTTP/REST.  The
    fragment would be what provides the query, but the semantics of HTTP are that fragments are processed
    on the client side, which is not what IDP necessarily calls for.  Though the server-oriented
    fragments could be mapped onto the query params in URLs.
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
-   Feature design notes
    -   Recursive fetch of a Plum and all its dependencies
        -   Requires implementation of Relation (one Plum depending on another) and Relation query for
            certain data types.
        -   Start by only supporting Relation query for a limited set of data types (BranchNode, DirNode).
        -   What's the difference between a content dependency and a metadata dependency?  This is meant
            to distinguish dependency trees consisting of only metadata from dependency trees that pass
            through content itself.  Examples:
            -   A branch consists of a sequence of BranchNode Plums.  Each BranchNode refers to its ancestor
                Plum (which is also a BranchNode or is "null") and a metadata Plum, as well as its content
                Plum.  The ancestor and metadata Plums are metadata, whereas the content Plum is content.
                Thus a metadata-only fetch would retrieve only the BranchNode and all its ancestors and all
                those nodes' metadata Plums, but not the content.  A content fetch would retrieve all metadata
                and content.
            -   A BranchNode represents a formally versioned state of the data that the branch is tracking.
            -   A BranchNode could/should also store the delta relative to the previous branch state.  How this
                would work when there is more than one ancestor node is not clear.  Some notion of a merge
                would be needed.
    -   Branching
        -   Basically analogous to git branches, except the kind of data being branch-tracked is arbitrary.
        -   Attributes of a BranchNode (each is a Plum, specified by its PlumHeadSeal):
            -   Ancestor BranchNode Plum -- the previous direct ancestor of this BranchNode in this branch.
            -   Metadata Plum -- should probably have a standard format, giving details about the BranchNode,
                similar to a git commit message.
            -   PosiDiff Plum -- diff that when applied to previous state of branch gets this state.
            -   NegaDiff Plum -- diff that when applied to this state of branch gets previous state.  Optional?
            -   Content Plum -- the content/state of the branch at this version; could be null in the case
                of formally versioned data, e.g. the state of a relational DB.
            -   MergeNode Plums (this would be a list of 0 or more BranchNode Plums (via their PlumHeadSeal values))
                This feature should be implemented later.
    -   Filesystem mapping
        -   Store Plum Head-s and Body-s as files in the filesystem using their seal values as their filenames.
            These should always be read-only, since they're immutable within the data model.
        -   Represent a DirNode on the filesystem as a directory, and create symlinks in that dir to the
            corresponding seal-named files, where the symlinks are named as the entries in the DirNode are.
        -   Add ability to create a read/write "working copy", just as in git, and "commit" a working copy
            to the Datahost, just as in git.  Will need to have some analog of the `.git` directory which
            tracks what branch the working copy is tracking, and against which diffs can be made.
        -   Partial commits (e.g. being able to select particular diff hunks to commit) would require more
            thought, but probably wouldn't be terribly difficult.
    -   Relational DB mapping (or just Key Value Store)
        -   The idea is to formally track the state of a DB (or single table within a DB?) using a kind of
            BranchNode.  The BranchNode won't actually store the content of the DB, but rather will just
            track the transactions on the DB, and the sequence of BranchNodes represent the sequential states
            of the DB.  This formal version tracking allows explicit fast-forwarding (and maybe eventually
            rewinding) of a copy of the DB to a given version.
        -   Basic operation is a CRUD operation on a DB.  This would be issued to some RPC or endpoint.
            The transaction would be encoded somehow and stored as the PosiDiff for the BranchNode that
            represents the new state of the DB.  A NegaDiff could be determined as well, if it's desired
            to be able to revert the DB operation, though implementation of such is nontrivial if DB
            table alterations are allowed.
        -   To replicate the state of a DB to a remote Datahost, that remote simply fetches the branch that
            tracks the DB, and then applies all the PosiDiffs from its current state to the head state.
            If the remote didn't want to keep the whole history of diffs, it could ask a server for a
            composed PosiDiff from a given branch state to another branch state.  In the case where DB
            entries are being modified more than inserted (e.g. in a turn-based game with a finite number
            of players, player location information would be updated more than inserted), and so the
            composed PosiDiff would be small compared to the full list of PosiDiffs.
    -   Layered filesystem mapping
        -   Similar to docker containers being layers of filesystems where the "upper" layers take precedence,
            a layered filesystem structure could easily be defined.
        -   A customized "open file for reading" function could be made to access the Plum that backs the
            desired file.
        -   Opening a file for writing would require a bit more thought, namely creating a file in a temporary,
            writable location, and then having an explicit "commit" operation, because once added to the
            datahost as a Plum, it becomes immutable.
    -   Layered caching scheme
        -   There are many different possible layers of caching, each with their own speed, size, and cost.
            For example:
            -   In-memory
            -   On local disk / in a local DB
            -   On nearby/fast server
            -   On distant/slow server
            -   In cold storage
        -   Each layer could have a caching policy which together determines the eviction policy:
            -   How much storage the cache is allowed to use.
            -   The cost of storage and/or retrieval.
            -   The prioritization of various items within the cache.
        -   There should be a reference data type (that has referential transparency) which refers to a
            resource in any arbitrary location, and that data type will handle the caching automatically.
            Obviously there are various details and complications for each level of caching, especially
            retrieving something from cold storage, but it can still be formally modeled.
        -   Analogously, there could be a non-transparent reference type (call it a pointer type) which
            has to be explicitly dereferenced in order to retrieve the data (i.e. cause it to be cached
            locally).
        -   Having documents that have transparent/opaque references would allow far richer data with
            greatly simplified access logistics.  Furthermore, "strong documentation" could use those
            references to refer to "official concept definition" Plums directly, instead of just through
            English text which has to be parsed and looked up by the human user.
    -   Filesystem / Relational DB use case: Regressive backup
        -   Having a fancy system is great, but it requires having fancy software that may have a finite
            lifespan, after which its fancy data is no longer usable.  Thus producing/keeping representations
            of the data in "regressive" form (e.g. mapped to a file hierarchy, or written to a DB) is good
            for hedging against loss of the ability to use the fancy software, as well as being able to
            back the data up into a form that is more plainly readable to a human.
