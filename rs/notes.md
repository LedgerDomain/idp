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

POC for an IDP-based DID (or DID-like) method (simplified so as not to go down a rabbit hole):
-   TODO later, once strongly authenticable, versioned data is implemented.
