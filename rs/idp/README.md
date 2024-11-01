# `rs/idp`

Rust SDK for Indoor Data Plumbing

See [a slide deck describing the Indoor Data Plumbing concept](https://docs.google.com/presentation/d/16azx22pCv_JvRslsRaSUPUSqTLQjyJ8oQZ-7-J3t1gw/edit?usp=sharing).

## Rust Crate Contents

-   `idp` -- Top-level module for crate, which imports the `idp_*` crates as submodules and has client/server integration tests.
    -   [`idp_core`](idp_core) aka `idp::core` -- Indoor Data Plumbing core data model and DB frontend, and feature-enabled GRPC client.
    -   [`idp_proto`](idp_proto) aka `idp::proto` -- Indoor Data Plumbing protobufs and GRPC functionality.
    -   [`idp_server`](idp_server) aka `idp::server` -- Indoor Data Plumbing server, using GRPC.

## To-dos

-   In client/server tests, use appropriate wait conditions instead of sleep statements to coordinate
    startup and shutdown of client/server.
-   Maybe use https://philcalcado.com/2018/11/19/a_structured_rfc_process.html within this git repo
    to document and moderate feature development process.
-   Each data type has an optional query method which can be used to query data out of that Plum.
-   Each data type has an optional relations method which can be used to determine which Plums
    a given Plum is related to.  Because some Plum bodies will be opaque data in certain contexts
    (e.g. sitting on server that is simply a datastore for clients), the Plum relation data could/should
    potentially be a separate part of the Plum, so that it can be packaged and sent to authorized
    clients so that they can do the tracking themselves.  For example, an IDP server is running in
    "lights out" mode, meaning that it's not tracking the relations itself, but will send the
    relation data to the client, which can track the relations and follow dependencies itself.
    Obviously this is slower than if the server can pre-compute relations on behalf of the client,
    but there are tradeoffs for "lights out".
    -   The server (when operating in server/client mode) effectively is a superuser, because it's
        privy to all the data.  It could do traversals of relation data for the client subject
        to the client's permissions (e.g. determining the full tree of dependencies for a given Plum).
-   The unit of data is the Plum, but higher order data structures (relational DBs, file hierarchies, etc)
    can be layered on top of the Plum layer.  This will be a conventional affordance to make IDP immediately
    understandable and usable.
-   Attempt to map the Plum Head/Body/Relations retrieval, as well as queries, onto URLs with HTTP/REST.  The
    fragment would be what provides the query, but the semantics of HTTP are that fragments are processed
    on the client side, which is not what IDP necessarily calls for.  Though the server-oriented
    fragments could be mapped onto the query params in URLs.
-   Why is this different from/better than existing HTTP/REST patterns?  Because the basic unit of data,
    the Plum (Head and Body) provides:
    -   Data authentication (via authenticable hashes)
    -   Unique addressing of content
    Tracking relations between Plums provides:
    -   Automatic fetching of dependencies, so that content can be automatically and completely replicated
        across peers.
    -   Construction of relational tables such as hyperlinks, @mentions, and #hashtags.
    Layering on specific data types, such as data types that contain Plum references, provide:
    -   DLT functionality (not just blockchain, but rather Merkle tree)
    -   Formal versioning (similar to git), which can even apply to DB states by tracking deltas to the DB.
-   How would the tracking of a relational DB work?
    -   State of the DB would be tracked formally via Plums that index:
        -   Previous state
        -   PosiDiffs (and potentially NegaDiffs) between states
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
            certain data types.  Or in general, a PlumRelations data structure that comes along with the
            head and body.  This way the relations don't have to be derived by intermediaries, since they
            might not be privy to the data from which the relations would be derived.
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
    -   Querying
        -   The idea is to be able to query specific data out of the hierarchy of known data types, e.g.
            -   DirNode: has entries with strings for names, so those strings form its "path" parameters.
                If there were a hierarchy of DirNodes, then a query could look like

                    dir0/downloads/file1.txt

            -   BranchNode: has ancestor, metadata, content, etc.  A query could look like

                    content/dir0/downloads/file1.txt

                where the branch content is the DirNode in the above example.
        -   Attempting to map this concept onto URLs (useful mainly for adoption; there is necessarily some
            loss in the mapping onto URLs), there can be server-side querying (query params) and client-side
            querying (fragment).  Such a URL could look like any of the following (noting that query params
            have to use %XX hex encoding for special chars, including `/`, but that fragment does not have to
            encode `/`):
            -   Server-side query

                    https://server.com/plum/<plum-head-seal>?q=dir0%2Fdownloads%2Ffile1.txt

            -   Client-side query

                    https://server.com/plum/<plum-head-seal>#dir0/downloads/file1.txt

            -   Half and half

                    https://server.com/plum/<plum-head-seal>?q=dir0%2Fdownloads#file1.txt

        -   I don't think the fragment query stuff would map directly into web browsers because they would
            lack the query implementation details.  But they're already limited in significant ways, and
            the point of IDP is to escape the limitations of existing patterns.

    -   DirNode
        -   Maybe allow symlinks as entries.  This feature could range in complexity:
            -   Simple, local symlinks only: an entry specifies the name of another entry, and it does the lookup,
                potentially nesting the lookups.  Cycles would need to be prevented.  E.g.
            -   Full symlinks: A symlink would effectively be replaced by a query string.  E.g. the symlink
                `stuff -> pics/ostrich.jpg` would indicate that a fragment query was needed to resolve it into
                a PlumHeadSeal (or `Box<Any>`, when that's implemented).  A DirNode by itself would not be able
                to support `../siblingdir`, because a DirNode by itself lacks the context needed to resolve it
                unambiguously.  And because a DirNode could an entry in potentially many other DirNodes, the
                resolution would depend on which parent DirNode it came through.  This makes sense within the
                context of a fragment query, but not otherwise.
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
        -   The unit of replication should be the DB (take SQLite for now).  All actions done to that DB
            would be recorded in the PosiDiff of the BranchNode that represents that DB state.  Actions
            are taken on the DB in the usual CRUD way, and the Plums for BranchNode are created as needed.
            This would include DB table alterations (though this is somewhat tricky, because usually
            migrations are done to bring the data schema up to date with the software, and in this case,
            that's not exactly true, you could potentially fast forward to a DB state that doesn't match
            your software's expected DB model schema).
            -   Example MVP app: Messaging with file attachments
                -   There should be a notion of a user with a UserId
                -   There should be a notion of publicly visible user profile info, which can mutate.
                    -   Track user profile data as a Plum
                -   There should be a notion of a message -- simply a Plum
                -   There should be a notion of an attachment -- also a Plum
                -   A message should be able to refer to users, attachments, and other messages via
                    some sort of @mention or #hashtag concept.  These relations should be tracked
                    formally and queryable.
                -   One goal should be that a client should be able to replicate the whole DB trivially
                    to its local store, and then access that DB as first class data locally, without
                    needing to talk to the server.
                -   Ideal goal would be to allow client to execute full SQL queries on its local DB replica,
                    so there's no need for abstraction inversions.
                -   DB tables
                    -   Public user directory, giving user profile info.  The columns would be:
                        -   User DID (unique, pseudonymous identifier)
                        -   Human-readable user name
                        -   PlumHeadSeal of active user profile Plum
                    -   Per-user message thread list -- these would be message threads that a given user is a
                        part of.  The columns would be:
                        -   Message thread ID (DID?)
                        -   Title
                        -   PlumHeadSeal of latest message in the thread; this would essentially be a BranchNode
                            whose metadata contains the Message thread ID and title (among other things)
                    -

                    -   Per-user attachment list -- these would be attachments that a given user has access to
                -   TEMP notes
                    -   Use ingestion of Plums to create relevant DB indexes/tables, e.g.
                        -   When ingesting a permissions/ACL Plum, update/insert relevant entries in/to an
                            access_control_lists table.  A "soft" resource identifier (e.g. a DID which is
                            not e.g. a hash of the PlumHead) would track its current state, which would
                            consist of the PlumHeadSeal of the resource itself, the PlumHeadSeal of its
                            ownership, and the PlumHeadSeal of its permissions.
                        -   When ingesting a Plum having ContentType MsgApp::UserProfile, create an entry in
                            a user_profiles table.
                        -   Generally, it should be possible to register a known type for tracking, so that
                            ingesting Plums of that type will cause relevant DB table entries to be updated/inserted.
                        -   When any PII is ingested, track it in a PII Plums table, so that an erasure
                            request can be easily complied with.
                -   Permissions scheme
                    -   Each user
                -   TODO START HERE: Need to define permissions scheme, and the permissions should be respected
                    in the DB replication.  I.e. a user that doesn't have access to particular elements of
                    the DB should not receive/replicate any of that data.  Maybe they can instead receive
                    only the CRUD operations for the entries they have access to, so that their local DB
                    tracks only the stuff they have permission for.  TODO start here by using a concrete
                    example and its permissions scheme.
    -   Permissions scheme
        -   TEMP notes
            -   Ownership
                -   Ownership of a resource should be fungible, but mediated by Plum-based records.
                    Could use a KERI-like record to log the sequence of ownership changes, which would
                    essentially be a proof that things have happened fairly.
            -   Permissions scheme "levels"
                -   Owner-world -- the owner is defined (who always gets full access) and the permissions for
                    the world are defined.  This is the "base case" of permissions.  Should this be represented
                    as a Plum?  Probably, for consistency.
                -   Could potentially make one that more directly models the POSIX permissions scheme.
                -   Access Control Lists -- access is mediated through an ACL, which defines who has what access,
                    and it can grant different levels of access (as well as access-granting) to different users
                    and groups.  But the owner still has full access.  An ACL is a Plum, and its access should
                    (at least for now) always use the owner-world scheme.
            -   By defining the permissions using a Plum, many Plums can "point to" the same permissions Plum,
                so resources could be grouped with respect to permissions.
    -   External storage of Plums
        -   Idea is that the Plum content itself might be private, and so it could be hosted and served
            from elsewhere, e.g. S3.  Plum uploads and downloads would happen directly to and from that
            target.
        -   Allow PlumHeads to be stored externally as well?
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
    -   There may be challenges in complete fetching of data when permissions come into play.  One way to
        handle this could be to implement some sort of "request access" feature.
-   IDP needs a description for what it actually does that's comprehensible to others and avoids the problem
    of someone immediately assuming it's something that exists already.
-   A design criteria of IDP would be to show how it reduces to many existing data patterns as particular
    applications, e.g.
    -   Permissioned filesystem
    -   Authenticable document system
    -   Git-like version control system
    -   Addressing of data for use across a network
    -   Data replication and caching system
-   Because IDP stores static, hash-auditable Plums whose data schemas are also static, yet applications
    inevitably need to apply migrations to data schemas, there should be a notion of data schema migration
    that can be (automatically) applied to Plums.
    -   This would necessarily come with a formal specification and versioning of the data schema (i.e. type)
        of each Plum, and the registration/retrieval of the canonical migrations (i.e. transformations) between
        schemas (i.e. types).  This would be analogous to DB migrations, and the design criteria should be that
        it should be usable transparently, not having to manually apply the migrations.  For example:
        -   A program might ask for a particular Plum (which is generally "User Profile" schema) and ask that
            it conform to "User Profile v1.2", but the stored Plum is "User Profile v1.0".  The Datahost would
            transparently apply the canonical migration that transforms a "User Profile v1.0" datum into
            "User Profile v1.2".
    -   The migrated Plum would have its own PlumHeadSeal, which should be authenticable, because the migration
        is supposed to be canonical (i.e. there is only one possible/definitive migration).  One might also
        just regenerate the Plum itself using the new data schema, if one wanted the Plum bodies in a uniform
        schema.
    -   This whole data schema migration would naturally be part of a lower-level system (in particular, sept),
        which manages a type runtime, and in particular, manages the various canonical type identifications/
        projects/embeddings, so that type conversions can be done automatically and/or transparently.
        These transformations could theoretically be provided universally as wasm modules using a wasm runtime.
-   Notes on some concepts
    -   What is the authority on a particular piece of data?
        -   Central authority (which could be a server or a user)
        -   Decentralized authority (having one of many different rule sets for how that's updated, ranging
            from modifications from whitelisted entities all the way to full consensus algorithms with
            anonymous entities)
    -   These notes are in service of a data model that is meant to be more peer-to-peer, and not client-server.
        Peers could work on their own local copies of data, and if they're the authority on that data, they
        can operate totally autonomously and definitively.  If they're not the authority on that data, they
        can work locally on a "proposal" for how the data is to be changed and submit that to the authority,
        be it on a server or on another peer.  A server could act as an cache/relay/intermediary for these
        interactions.
    -   The idea here is that application logic could be implemented in one place (the "peer") instead of
        across a client/server boundary, and instead let IDP handle the data propagation.  This would be
        closer to a git data model.  A server would be a peer that functions in the following ways:
        -   Operate as a proxy/relay for other peers
        -   Operate as a cache/origin for data, including maintaining caches for indexes/views into relational data.
        -   Operate as a data store for peers/clients that don't have long-term persistent storage (i.e. web browers)
        -   Could operate as the authority for certain data (or on behalf of some other authority), which is
            just the special case where it's acting as a traditional server.
    -   The challenge to this approach is that it's harder to push code updates to an IOS app than to a server.

## To-don'ts (I.e. Done)

-   Using features to enable client and/or server code generation within `idp_proto` and generally in `idp`.
    This way no redundant/unused code is generated.
