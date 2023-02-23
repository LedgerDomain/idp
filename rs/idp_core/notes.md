# Design Notes

2023.02.13
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

2023.02.15
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

2023.02.16
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
    -   created_at_o (should this be called plum_created_at_o?)
    -   plum_body_content_length
    -   plum_body_content_type
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

2023.02.22
-   Datahost metadata notes
    -   There should be a table in the datahost DB containing a single row where metadata for the Datahost is stored, e.g.
        -   Some unique identifier (e.g. a UUID V4)
        -   Name?
        -   Creation timestamp
-   "Broken dependency" tracking

    The idea is that precise tracking of complete dependency graphs should be done, so that incomplete dependency graphs and their missing pieces can be identified efficiently.  The PlumBody isn't actually necessary for this computation, since the PlumRelations is a separate piece of data that can go along with the PlumHead.  Thus dependency info is a kind of metadata not requiring knowledge of the PlumBody.

    Ways that a Plum can have incomplete dependencies
    -   If the Plum is missing its PlumRelations, then it must assume that it is missing dependencies. Thus, for the rest of these items, assume the PlumRelations is present.
    -   For a given DAG-type relation (which basically has to be the "strongest" kind of dependency; which would be defined to be "dependencies that are required to understand this Plum", e.g. image files that are referred to in a document), the number of expected direct dependencies of that type is known via the PlumRelations, and can be counted in the DB.  However, this is a transitive property; if a dependency has an incomplete dependency graph, then the original Plum should be considered to have an incomplete dependency graph.

        The number of expected transitive dependencies and number of present transitive dependencies could be tracked for each Plum, and if present is less than expected, then the dependency graph is incomplete.  Then it's a simple matter to identify which Plums need to be recursed to to retrieve the missing dependencies.

        If this is tracked with an i32 (DBs typically store signed ints), the max number of transitive dependencies is a bit more than 2 billion, which could conceivably be achieved in some cases for a very large collection of indexed data.  Thus i64 would certainly be more than enough.  So this would add two i64 columns to each Plum (stored in the plum_heads table).  If the PlumRelations for a Plum aren't present, then the plum_heads row should contain NULL for expected and present values, since neither can be computed.  Once the PlumRelations is obtained, then expected and present can be computed recursively.  PlumRelations should store the expected number of transitive dependencies, since computing it directly would require having all PlumRelations.

        Could "incomplete" vs "complete" be tracked using a single bit?  Certainly transitioning from complete (1) to incomplete (0) is easy.  However to go from incomplete to complete requires verifying that each dependency is present and each dependency has its complete bit marked.  The advantage of this is that each relation type can be tracked.  This probably also lends itself well to scanning the DB and identifying which Plums are incomplete for any subset of relation types (this involves a bitwise AND with a relation type bitmask and then a comparison with that bitmask; if they're not equal, then the Plum is incomplete in the bits that are different).
