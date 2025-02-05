# Design Notes

## 2023.02.13
-   Need to rethink how sealed PlumHead-s, PlumRelations-es, and PlumBody-s are stored.  Apparently protobuf isn't necessarily deterministic in its serialization (https://gist.github.com/kchristidis/39c8b310fd9da43d515c4394c3cd9510), so it's not possible to reconstruct a protobuf-serialized PlumHead, PlumRelations, or PlumBody and guarantee that it produced the same bytes, and therefore the seal (hash of the sealed value) can't be verified. Possible solutions:
    1.  Store the sealed value directly, in whatever serialization format it's in.  Then there's no issue of re-serializing it in order to verify the seal.
        -   Benefits
            -   Admits any serialization format, even ones that don't have deterministic serialization, because the sealed value is a fixed, particular serialization.  In particular, JSON would even be usable here.
            -   Transmission of sealed values (heads, relations, bodies) is simple and direct, and doesn't require a re-serialization step.
            -   Datahosts could more easily self-audit:
                -   Check the seals on all data
                -   Verify that the sealed data (when deserialized) matches the unpacked data in the DB.
        -   Drawbacks
            -   If sealed values are stored in addition to unpacked data in the DB, then that potentially doubles the storage necessary.  Or if not all unpacked data is stored in the DB, then portions would need to be retrieved from the sealed values when needed (e.g. the PlumBody content).  This could be hard because not all serialization formats support efficient queries on embedded values, instead requiring full deserialization.
    2.  (2023.02.22: This one was chosen) Store the sealed value directly, and use a particular deterministic process for computing/verifying the seal.
        -   This requires the following values to be inside the sealed value:
            -   Specification of the seal algorithm (e.g. hash/HMAC/signature)
            -   Enumeration of the components of the sealed value, specifying deterministically how they are mapped into a byte sequence to be fed into the hash/HMAC/signer.
        -   Benefits
            -   There's no need to store the serialized form of the value, and therefore no redundant storage is needed.  Plum bodies (which could be truly huge) could be stored in the filesystem, separate from the other associated values, for example.
            -   Plum body content can be read directly (especially from the filesystem), using first-class file operations (e.g. seek).
            -   Plum body content, if stored in the filesystem, can be easily mapped into virtual directory structures to provide a filesystem-based view into structured data.
            -   Serialization format (e.g. for transmission or other storage) doesn't matter, and therefore this is compatible with any lossless serialization format.
            -   Self-audit can happen in the DB that stores the unpacked values directly, instead of needing to audit serialized blobs and then verify that they match the unpacked values.
        -   Drawbacks
            -   This involves more complexity in producing and verifying seals on values, since it involves a specific procedure to produce a deterministic byte sequence to be used to hash/HMAC/sign.
            -   Certain datatypes are trickier to serialize deterministically, e.g. hash maps, so they would require more complex procedures to handle.

        Note that there are only a finite number of types for which this deterministic sealing process would need to be defined for:
        -   PlumHead
            -   Seal specification for seal over PlumHead (e.g. which hash function/HMAC/signature/etc; including nonce/key/etc)
            -   Body seal
            -   Body content type
            -   Body length
            -   Relations seal (optional; omitted if there are no formal relations)
            -   Owner ID (optional; this ties into a permissions scheme, so need to think about this)
            -   Creation timestamp (optional)
            -   Unstructured metadata (optional; these are just bytes that can be used in any way)
        -   PlumRelations
            -   Seal specification for seal over PlumRelations (e.g. which hash function/HMAC/signature/etc; including nonce/key/etc)
            -   Plum relation flags mapping pairs (assume lexicographical ordering to produce deterministic result)
                -   Target PlumHeadSeal
                -   Relation flags (as bitmask)
        -   PlumBody
            -   Seal specification for seal over PlumBody (e.g. which hash function/HMAC/signature/etc; including nonce/key/etc)
            -   Body content

    Option 2 is the winner, if for nothing else than its friendliness with the filesystem (for big files and for interfacing with other tools in the style of `git`).

-   Define a "seal specification" structure which specifies a hash function + nonce, HMAC + key, signature scheme + pub key, etc.  Rust pseudocode:

        enum SealSpecification {
            // This is necessary up front.
            Hash {
                hash_function_name: HashFunctionName,
                nonce_o: Option<Nonce>,
            },
            // This could be added later
            HMAC {
                hmac_function_name: HMACFunctionName,
                key: HMACKey,
            },
            // Maybe later add digital signature, though perhaps that belongs as a formal Plum itself.
        }

        TODO: Start here -- SHA256 and SHA512 (SHA-2) are prone to length-extension attacks (even with nonce), so it may be advisable to skip them entirely and only use HMACs.  On the other hand, simple storage in a low-stakes environment doesn't really call for an HMAC's level of protection, and in order to get deduplication of body content, a hash with no nonce is preferable.

        enum HashFunctionName {
            SHA256,
        }

        enum HMACFunctionName {
            SHA256,
        }

## 2023.02.15
-   Noticed that plum_relations_nonce_o is not being stored in the DB, and so it's not possible to form a PlumRelations from the content of the DB.  The way to fix this is:
    -   Have two tables for relations data:
        -   plum_relations
            -   plum_relations_rowid : primary key (auto increment)
            -   plum_relations_seal : PlumRelationsSeal (index on this also)
            -   relation_mappings_count : uint counting the number of specific relations
            -   plum_relations_nonce_o : optional nonce
            -   source_plum_head_seal : unique PlumHeadSeal for the Plum that this PlumRelations pertains to; this could also simply be a foreign key into a plum_head_seals table. 2023.02.18 NOTE: The head seal can't be put into the PlumRelations, since PlumHead contains the PlumRelationsSeal, so there's a cyclic seal dependency, which is infeasible. Thus it should be the PlumBodySeal instead, since that produces a properly directed dependency graph.
        -   plum_relation_mappings
            -   plum_relation_mappings_rowid : primary key (auto increment)
            -   plum_relations_rowid : the foreign key identifying which relations this is for
            -   target_plum_head_seal : PlumHeadSeal for the Plum that this PlumRelations entry targets; this could also simply be a foreign key into a plum_head_seals table.
            -   There should be a unique constraint on (plum_relation_mappings_rowid, target_plum_head_seal)
    -   To iterate over all plum relation mappings for a given PlumHeadSeal:
        -   Look up plum_relations_rowid for the given source_plum_head_seal.
        -   Select rows from plum_relation_mappings with the looked-up plum_relations_rowid.
    -   To recover the PlumRelations from the DB for a given PlumRelationsSeal:
        -   Select the row in plum_relations with plum_relations_seal.
        -   Select rows from plum_relation_mappings with the looked-up plum_relations_rowid.
        -   Construct PlumRelations from these results.
-   Actually, it shouldn't be possible to not have a PlumRelations for a Plum.  Even if there are no relations, it should be specified.  This allows for simpler logic, and less ambiguity.
-   Plan for implementation
    -   Because plum_relations table is already missing plum_relations_nonce_o, those relations nonces are gone and not recoverable, thus the existing records in the DBs can't be migrated (unless it's assumed that the nonces are all None).  This could be a decent reason to switch to sqlx, since starting over doesn't matter at that point.
    -   To switch to sqlx:
        -   Create a `[Datahost?]Storage` trait in the idp_core crate.
        -   Create a (sub)crate called `idp_storage_sqlite` (or `idp_datahost_storage_sqlite`?) which implements the trait using sqlx with a SQLite backend.

## 2023.02.16
-   Should the body length and content type go in PlumHead or PlumBody?
    -   In PlumHead
        -   Benefits
            -   More-easily accessible metadata.
            -   More complete picture over data for which the Datahost only has the PlumHeads.
        -   Drawbacks
            -   Less opportunity for privacy/non-disclosure.
            -   PlumBody is not independently understandable.
    -   In PlumBody
        -   Benefits
            -   More opportunity for privacy/non-disclosure.
            -   PlumBody is independently understandable.
        -   Drawbacks
            -   Less-easily accessible metadata.
            -   Less complete picture over data for which the Datahost only has the PlumHeads.
-   Disclosure of body length and content type in PlumHead could be optional, based on the Plum's intended use cases.  This would add some redundancy (two fields) and would require additional checks.  This whole issue is operating under the assumption that it is desired that PlumBody be independently understandable.
-   A similar argument could be made that the owner_id_o, created_at_o, and potentially metadata_o fields of PlumHead should go in the body as well, and that PlumHead is only meant to facilitate sealing the PlumRelations and PlumBody, making Plums addressable, and allowing relations on Plums when you don't have the PlumBody-s.

    Is "owner ID" the right thing to store?  Shouldn't it be "author ID", since "owner" suggests that it's not the same as "author" and has to do with permissions and potentially can be changed.

    Another approach could be to separate out the metadata entirely, i.e. create a PlumMetadata type containing:
    -   plum_metadata_nonce_o
    -   author_id_o (should this be called plum_author_id_o?)
        -   If this field is to represent a cryptographic identity, then it should probably come with a signature, so that authorship is actually verifiable, and not forgeable.  What would the signature be over?  It couldn't be the PlumMetadata or the PlumHead (since that would create an infeasible cyclic definition), so the only thing(s) remaining would be the PlumRelations and PlumBody.  One option would be to have the signature be in the PlumHead itself, so that the signature could be over the PlumMetadataSeal, PlumRelationsSeal, and PlumBodySeal.  On the other hand, maybe authorship should be a higher-order concept, and related to the plum via a plum representing an authorship claim.
    -   plum_body_content_length (potentially redundant with PlumBody)
    -   plum_body_content_type (potentially redundant with PlumBody)
    -   Optional, additional metadata of some form, which is either:
        -   Option 1
            -   additional_metadata_content_length (could be required but defaults to 0)
            -   additional_metadata_content_type (could be required but defaults to empty)
            -   additional_metadata_content (could be required but defaults to empty)
        -   Option A
            -   additional_metadata

    And then PlumHead would consist of:
    -   plum_head_nonce_o
    -   plum_metadata_seal (required; its fields however are optional)
    -   plum_relations_seal_o (should this be required, where it can simply refer to an empty relations?)
    -   plum_body_seal

## 2023.02.22
-   Datahost metadata notes
    -   There should be a table in the datahost DB containing a single row where metadata for the Datahost is stored, e.g.
        -   Some unique identifier (e.g. a UUID V4)
        -   Name (of datahost)?
        -   Creation timestamp
        -   Creator/owner/root user?
-   "Broken dependency" tracking

    The idea is that precise tracking of complete dependency graphs should be done, so that incomplete dependency graphs and their missing pieces can be identified efficiently.  The PlumBody isn't actually necessary for this computation, since the PlumRelations is a separate piece of data that can go along with the PlumHead.  Thus dependency info is a kind of metadata not requiring knowledge of the PlumBody.

    Ways that a Plum can have incomplete dependencies
    -   If the Plum is missing its PlumRelations, then it must assume that it is missing dependencies. Thus, for the rest of these items, assume the PlumRelations is present.
    -   For a given DAG-type relation (which basically has to be the "strongest" kind of dependency; which would be defined to be "dependencies that are required to understand this Plum", e.g. image files that are referred to in a document), the number of expected direct dependencies of that type is known via the PlumRelations, and can be counted in the DB.  However, this is a transitive property; if a dependency has an incomplete dependency graph, then the original Plum should be considered to have an incomplete dependency graph.

        The number of expected transitive dependencies and number of present transitive dependencies could be tracked for each Plum, and if present is less than expected, then the dependency graph is incomplete.  Then it's a simple matter to identify which Plums need to be recursed to to retrieve the missing dependencies.

        If this is tracked with an i32 (DBs typically store signed ints), the max number of transitive dependencies is a bit more than 2 billion, which could conceivably be achieved in some cases for a very large collection of indexed data.  Thus i64 would certainly be more than enough.  So this would add two i64 columns to each Plum (stored in the plum_heads table).  If the PlumRelations for a Plum aren't present, then the plum_heads row should contain NULL for expected and present values, since neither can be computed.  Once the PlumRelations is obtained, then expected and present can be computed recursively.  PlumRelations should store the expected number of transitive dependencies, since computing it directly would require having all PlumRelations.

        Could "incomplete" vs "complete" be tracked using a single bit?  Certainly transitioning from complete (1) to incomplete (0) is easy.  However to go from incomplete to complete requires verifying that each dependency is present and each dependency has its complete bit marked.  The advantage of this is that each relation type can be tracked.  This probably also lends itself well to scanning the DB and identifying which Plums are incomplete for any subset of relation types (this involves a bitwise AND with a relation type bitmask and then a comparison with that bitmask; if they're not equal, then the Plum is incomplete in the bits that are different).

-   Potential POCs
    -   Non-versioned state, addressed via path.
        -   A path simply maps to a PlumHeadSeal.
        -   There should be a notion of permissions.  Probably those permissions are also simply state-based, no Plum necessary; this would be proportional to what this "non-versioned state" is.  Though perhaps permissions' versionedness can be a separate thing than the state's versionedness.
    -   Versioned data tracking (equivalent to a branch in git).  This would involve having a kind of service that handles state update, and could warrant fleshing out the state/service abstraction.  Operations:
        -   Get PlumHeadSeal of current head BranchNode (the user can pull that Plum as a separate operation in order to retrieve the entire branch history).
        -   Fast-forward branch to a given PlumHeadSeal; equivalent to `git push` (the user must have pushed that Plum in order for the server to have the necessary updated branch history).
        -   Rewind branch to a given PlumHeadSeal in its history.
        Realistically, these should be permissioned operations, but for now, no need.

        The state of these needs to be tracked, the most obvious would be in the same DB as the plums, though in the future it might make sense to track that separately, especially as arbitrary, plug-in stateful services are implemented.  Needed table(s):
        -   path_states (these store a PlumHeadSeal for a given path); columns:
            -   path_states_rowid (integer PK)
            -   path (string; let's disallow '/' for now, because paths for a directory hierarchy is different)
            -   plum_head_seal (PlumHeadSeal; this is the state of this path)
            -   something for permissions (TODO: there should be a history of permissions encoded in a plum, probably, but practically speaking it would likely also be tracked in the DB for efficient queries)
    -   Digital signatures
        -   Null hypothesis would be that a digital signature is just a piece of data like any other and would be stored as a Plum, and reasoned/handled accordingly, though it would introduce a new relation type "signature on" (or something to that effect).
        -   Potentially a digital signature could be used as a Seal.  This would be primarily useful in proving authorship.
    -   A strong DID method (or something analogous but simpler), capable of historical DID doc resolution. This would also be stateful.
        -   The DID itself would be a URL to a state on a particular server, e.g.

                did:idp:example.com/path/to/thing

        -   The DID doc would just be the expected JSON document, but it would be stored indirectly via versioned datatypes, e.g.
            -   did:idp:example.com/path/to/thing -> PlumHeadSeal of head BranchNode (call this XYZ)
            -   BranchNode XYZ
                -   content -> PlumHeadSeal of DID doc
                -   metadata -> Relevant signatures authorizing the update to the DID doc, relevant timestamps, etc.
                -   ancestor -> PlumHeadSeal of BranchNode for previous state of DID doc

        -   The "endpoint" idp://example.com/path/to/thing is really a stateful service, and should have its own API, which isn't necessarily the same as HTTP or HTTP's CRUD-like operations.  For example, historical DID document resolution should be possible via some particular API call like "give me the PlumHeadSeal of the BranchNode active during timestamp X".  Then, there are two useful situations:
            -   The user retrieves (or has already retrieved) the entire branch history of the DID and has verified its correctness (all seals, signatures, and any other constraints).
            -   The user could be using a proxy IDP Datahost which they trust to have retrieved and verified the entire branch history of the DID, and user can simply retrieve the specified BranchNode and its contents.

        -   The "endpoint" also has an update operation which simply checks that the update is well-formed and has all the necessary signatures.  This well-formedness and verification of necessary signatures functions as authorization.
        -   Ideally the document update and verification logic is generic, not DID-specific.  This would involve some sort of generic "strong, versioned document" abstraction which is simply used to store DID documents.  Similarly, revocation could be handled this way without any logic specific to revocation.

## 2023.04.25

Notes on content type and content encoding
-   Want to define standards-compliant content type for custom IDP data types, e.g. `DirNode` and `BranchNode`
-   Want to define standards-compliant content encoding values for data used in IDP, e.g. `msgpack` or whatever.  Or does this "first-level" encoding (i.e. not the later compression layer) go as part of the content type?
-   Content type (reference: https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types)

        type/subtype;parameter=value

    Examples:
    -   text/plain
    -   text/plain;charset=us-ascii
    -   text/plain;charset=utf-8
    -   text/html
    -   image/png
    -   image/jpeg
    -   application/octet-stream
    -   application/jwt
    -   application/pdf

    Content type is actually a conflation of the semantic type (i.e. what the data is/means) and its serialization format (how it's represented as a byte stream).  Thus the content type should imply the serialization format.

    Content encoding represents additional layers of encoding (typically compression, though base64-encoding is a commonly used encoding that actually inflates the data).

    After a convo with ChatGPT 4, I think the following would be appropriate; `x.` is still allowed (it's different than the deprecated `x-` prefix), and indicates "experimental":
    -   application/x.idp.BranchNode+msgpack
    -   application/x.idp.BranchNode+json
    -   application/x.idp.BranchNode+proto2
    -   application/x.idp.DirNode+msgpack
    -   application/x.idp.DirNode+json
    -   application/x.idp.DirNode+proto2

    The plus sign is used to distinguish the semantic type from the serialization format.

References:
-   https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types

## 2023.05.01

More on content format
-   I have implemented `ContentClass`, `ContentFormat`, and `ContentEncoding`.  Together, `ContentClass` and `ContentFormat` imply a `ContentType` (in the sense of the HTTP `Content-Type` header).  `ContentClass` indicates the semantic type of data, and `ContentFormat` determines the serialization format.  `ContentEncoding` is a sequence of optional, additional encodings to apply to the serialized data (e.g. gzip, deflate, base64, etc).  The initial implementation has a couple of drawbacks:
    -   Each data type has to implement a serialization method which will serialize the data using a specified `ContentFormat`.  This is obviously not ideal because it just means more boilerplate, and in 95% of cases for structured data, the impl of `serde::Serialize` is what should be used, with different `ContentFormat` values determining which serde serializer should be used.
    -   There isn't a 1-to-1 correspondence between `ContentFormat` and serde serializers.  Examples:
        -   `ContentClass`: `text/plain` and `ContentFormat`: `charset=us-ascii` is simply an ASCII string (all chars are guaranteed to be ASCII chars), and so there's no associated serde serializer.  Analogously `ContentFormat`: `charset=utf-8` is a UTF8 string.
        -   `ContentClass`: `image` has many possible `ContentFormat` values, e.g. `png`, `jpeg`, etc.  However, none of those formats correspond to a serde serializer.
    -   Some data types have an implied or default `ContentFormat`, or the system/user might have a preferred `ContentFormat` for generating new data (e.g. they might want it all to be human-readable JSON, or machine-readable protobuf or CBOR), but the initial implementation requires specifying `ContentFormat` for everything.
-   In order to address the non-1-to-1-ness of serialization formats and serde serializers, there could be a formal notion of `SerializationFormat` which defines serialization and deserialization of data.  Each kind of serde serializer should be a `SerializationFormat` which applies to structured data (for which `serde::Serialize` and `serde::Deserialize` are implemented).  But there should also be data-type-specific `SerializationFormat`s, e.g. corresponding to `charset=us-ascii`, `charset=utf-8`, `png`, `jpeg`, etc.  There should also be a "none" (or "raw"?) `SerializationFormat`, e.g. for `application/octet-stream` or other `ContentClass`es which don't have an explicit format.
