# High Level Design Notes the IDP Project

## 2023.05.02

POC for `libp2p`-based IDP peer
-   Instead of a client/server architecture, use `libp2p` to allow direct peer-to-peer communication.
-   `libp2p` has its own peer discovery mechanism, but the IDP peer should be able to specify the specific address of the peer.
-   It should support all the same features as client/server (e.g. push, pull, etc).
-   Ability to specify a communication intermediate (I forget what the `libp2p` term is) so that mobile devices could run a peer that have a cloud-based communication intermediate to receive messages and send push notifications indicating that there's a message.
-   Current IDP implementation uses protobufs for its API types (including in its internals).  Maybe implementing a `libp2p` IDP peer would warrant separating those out into their own real Rust types finally (this would also address some issues, such as the impl of `Debug` for things like hashes already being specified by `prost` to print an array of decimal-formatted bytes instead of defined by IDP to print something more reasonable like a compact string of hex digits or base64).

POC for "watch"-based updates
-   Idea here is that a client/peer could indicate to a server/other peer that it's watching a given `PathState` and wants to receive notifications of any changes to it, so that it can automatically pull those changes and stay in sync as the changes happen.  This way, that data can already be present locally for when the client/peer needs to do something with it.  For example, pulling the DID documents for a set of specified DIDs that it knows it interacts with frequently, so when those DIDs are resolved, it can be done using only local data and not incur a network operation.
-   A client/server-friendly version of this could still involve the ability for a client to add watches, but then simply poll to ask about updates.

POC for "strongly authenticable", non-versioned data
-   A `PathState` would have an owner (which later could be group of owners), and updates to that `PathState` would require the update to be signed by that owner.  This could be done using a `SignedRef` Plum which contains the PlumHeadSeal of the target data, the ID of the owner, and a signature by the owner over that PlumHeadSeal (the signature should be by the owner of the current `PathState`'s `SignedRef`).  Perhaps some other metadata as well.
    -   The owner of the new `SignedRef` could be different, indicating a handoff of ownership.  Maybe there are rules for what's allowed in that category.
-   MVP of this could use `did:key` as the identity layer for prototyping.
-   Form of `SignedRef`
    -   Signer: DID (or other identifier, perhaps identifying a group)
    -   Nonce -- should be required
    -   Signature timestamp -- required
    -   Additional content -- optional
    -   Referent: PlumHeadSeal -- this is the PlumHeadSeal of the Plum that this SignedRef refers to.
    -   Signature -- should be over the hash of each of the above attributes.
-   A `SignedRef` should have a special kind of relation to its Referent.  Certainly it should be a content dependency, but it should also have a "signature on" relation.
-   Can this "strongly authenticable" feature be decoupled with versioning?  So that:
    -   The "outer" Plum type is `SignedRef`
    -   The "inner" Plum type is `X` (for some `X` that potentially has rules governing it)
    -   Validation of updates to the path start with validating the rules regarding the current and the new `SignedRef` and then invoke any rules for governing the inner type `X`.
    -   Are these actually separable, or might there be rules that involve both `SignedRef` and `X`?
    -   If this is decouplable, then this greatly simplifies things.  This should be a design criteria.

POC for "strongly authenticable", versioned data
-   This should be compatible with the non-versioned data form.
-   Versioned data is simply a `BranchNode`, pointed to by a `SignedRef`.
-   Obviously the rules for validating the updates regarding `SignedRef` and `BranchNode` still apply, but there might be more rules that involve both of them simultaneously.
-   Some random ideas:
    -   There should be a general-purpose `Governor` which controls a `PathState` where all operations are signed by an "owner" (or otherwise authorized) entity, so that the `PathState` (and any history it has via e.g. `BranchNode`) is cryptographically verifiable.  Note though that this requires having an identity layer.
    -   Maybe more basic would be that the whole branch history of the strongly authenticable is identified by its initial PlumHeadSeal, to use the `PlumRelations` to find potential successor `Plum`s, and strict rules defining which the "real" successor `Plum` is.  The challenge here is having a deterministic way to define the successor (this is analogous to the double-spend problem).  The existence of this primitive would suggest having a "pull" operation which does this traversal while also checking constraints and signatures on everything.  If this were a primitive of IDP, then something like a DID method or a revocation method could be easily implemented.  If the origin IDP server/peer is considered the authority on the state of this versioned data, then it's analogous to changing the history of a git branch (a sometimes valid operation), and in that case, there's no double-spend problem.

POC for "strongly authenticable" data, decoupled from versioned vs non-versioned consideration
-   A `PathState` would have some definition of the owner of the path.  Perhaps `PathState` itself would get an `owner_o` field which optionally can specify the owner.  Or, this "owner" could be put into a separate Plum entirely in order to further separate considerations; this might even make it possible to have a commutative diagram.
-   The `PathState` would be updated using various formal edits, each of which are signed by the owner (if there is an owner; otherwise they're "bare" edits).  Note that the path itself is omitted from the edits so that it's possible to rename the path itself.
    -   create PathState
        -   owner_o: Option<DID>
        -   current_state_plum_head_seal: PlumHeadSeal
    -   UpdatePlum edit -- this edit is reversible simply by reversing the PlumHeadSeal values:
        -   previous_state_plum_head_seal: PlumHeadSeal
        -   new_state_plum_head_seal: PlumHeadSeal
    -   ChangeOwner edit -- NOTE that in order for this to be reversible, both the previous and new owners would need to sign the edit Plum (this implies that either (1) the update must be able to take multiple signatures, or (2) a PlumSig needs to be able to store multiple signatures; for now, let's just go with 1 for simplicity):
        -   previous_owner_did_o: Option<DID>
        -   new_owner_did_o: Option<DID>
    -   Tombstone edit
        -   previous_state_plum_head_seal: PlumHeadSeal
-   Because the `PathState` may be governed by something more (e.g. a branch is governed by rules regarding branch updates), once the basic rules defined above are satisfied, the type-specific rules are checked.  Maybe this can be achieved in general by using a layering of "governors".  In the case of strongly authenticable versioned data (i.e. using `BranchNode`), this would first invoke the "strongly authenicable data" governor, and then the "branch" governor.  This layering scheme can't capture more complex workflows that would be represented by a DAG of governors, and it's not clear if there are any obvious workflows that would require a DAG of governors.
-   In order for the "strongly authenticable data" feature to still work once DID key rotation is available, it will be necessary to put timestamps in the edits.  Though of course also historical DID document resolution would be necessary as well.
-   Does this "strongly authenticable data" scheme allow for historical data resolution?  Yes -- the state could be unwound until the desired timestamp is achieved, and then that data returned.  However this is an O(n) operation, and it would be better if there were a table of updates so that the query could be done quickly.  Because the table must be constructed from the edits, and the content of the table is not intrinsically immutable (it could be modified after the fact), it should be considered to be something that the consumer of the historical data should do themselves within their own data host.  This reconstruction should happen automatically within the operation of the datahost when one fetches a `PathState` and its history.
-   Does this "strongly authenticable data" scheme allow for the branch functionality of a rewind + fast-forward?  I.e. to rewrite part of history?  This brings up the question of exactly how mutable history should be for strongly authenticable data.  On one end, it should not be rewritable (i.e. you can only edit into the future, never unwind edits).  On the other, it should be rewritable like git branches.  There are use cases for both.  Perhaps this should just be a parameter of the kind of versioning that should be used (i.e. immutable history like a (micro)ledger, or rewritable like a git branch).  Either way, this puts the rewritability of history into the consideration of the underlying governor, not the strongly authenticable data governor.
-   Attempt to decouple strongly authenticable data (SAD) from branch-versioned data
    -   Actions for SAD
        -   Set the initial owner and state for a SAD
        -   Take an action on the the state of a SAD (this should invoke the "inner" governor -- examples of inner governor: (1) none, (2) no-rewrite, single-timeline history, (3) branch-based data)
        -   Modify the owner of a SAD (should require sigs from prev and new owners)
        -   Tombstone a SAD (i.e. disallow further actions)
    -   Actions for branch-versioned data
        -   Create a branch (BranchNode has no ancestor)
        -   Delete a branch
    Design criteria: Have the PlumSigs be a separate layer so that an existing branch could later be signed.

POC for an IDP-based DID (or DID-like) method (simplified so as not to go down a rabbit hole):
-   TODO later, once strongly authenticable, versioned data is implemented.
