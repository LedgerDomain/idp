-- Drop existing tables, since this is a clean start.

DROP INDEX IF EXISTS paths;
DROP TABLE IF EXISTS path_states;

DROP INDEX IF EXISTS plum_body_seals;
DROP TABLE IF EXISTS plum_bodies;
DROP TABLE IF EXISTS plum_relation_mappings;
DROP INDEX IF EXISTS plum_relations_seals;
DROP TABLE IF EXISTS plum_relations;
DROP INDEX IF EXISTS plum_head_body_references;
DROP INDEX IF EXISTS plum_head_seals;
DROP TABLE IF EXISTS plum_heads;

-- Recreate all tables.

CREATE TABLE plum_heads (
    -- Primary key
    plum_heads_rowid INTEGER NOT NULL PRIMARY KEY,

    -- Other DB-oriented attributes.
    row_inserted_at BIGINT NOT NULL,

    -- This is the "real key" for a PlumHead, i.e. its globally unique ID.
    plum_head_seal BLOB NOT NULL,

    -- PlumHead attributes.
    plum_head_nonce_o BLOB,
    plum_metadata_seal BLOB NOT NULL,
    plum_relations_seal BLOB NOT NULL,
    plum_body_seal BLOB NOT NULL,

    UNIQUE(plum_head_seal)
);

-- This index is used so that plum_head_seal lookups are fast.
CREATE INDEX plum_head_seals ON plum_heads(plum_head_seal);

-- These indexes are used so that plum_head -> plum_metadata, plum_head -> plum_relations, and
-- plum_head -> plum_body reference counting is efficient.
CREATE INDEX plum_head_metadata_references ON plum_heads(plum_metadata_seal);
CREATE INDEX plum_head_relations_references ON plum_heads(plum_relations_seal);
CREATE INDEX plum_head_body_references ON plum_heads(plum_body_seal);

CREATE TABLE plum_metadatas (
    -- Primary key
    plum_metadatas_rowid INTEGER NOT NULL PRIMARY KEY,

    -- Other DB-oriented attributes.
    row_inserted_at BIGINT NOT NULL,

    -- This is the "real key" for a PlumMetadata, i.e. its globally unique ID.
    plum_metadata_seal BLOB NOT NULL,

    -- PlumMetadata attributes.
    plum_metadata_nonce_o BLOB,
    -- This is in Unix nanoseconds (nanoseconds since Unix epoch, which is 1970-01-01 00:00:00 UTC).
    plum_created_at_o BIGINT,
    -- The plum_body_content_* columns must either all be NULL or all be non-NULL. See CHECK clause below.
    plum_body_content_length_o BIGINT,
    plum_body_content_class_o TEXT,
    plum_body_content_format_o TEXT,
    plum_body_content_encoding_o TEXT,
    -- The additional_content_* columns must either all be NULL or all be non-NULL. See CHECK clause below.
    additional_content_length_o BIGINT,
    additional_content_class_o TEXT,
    additional_content_format_o TEXT,
    additional_content_encoding_o TEXT,
    additional_content_byte_vo BLOB,

    UNIQUE(plum_metadata_seal),
    CHECK(
        plum_body_content_length_o IS NULL AND
        plum_body_content_class_o IS NULL AND
        plum_body_content_format_o IS NULL AND
        plum_body_content_encoding_o IS NULL
        OR
        plum_body_content_length_o IS NOT NULL AND
        plum_body_content_class_o IS NOT NULL AND
        plum_body_content_format_o IS NOT NULL AND
        plum_body_content_encoding_o IS NOT NULL
    ),
    CHECK(
        additional_content_length_o IS NULL AND
        additional_content_class_o IS NULL AND
        additional_content_format_o IS NULL AND
        additional_content_encoding_o IS NULL AND
        additional_content_byte_vo IS NULL
        OR
        additional_content_length_o IS NOT NULL AND
        additional_content_class_o IS NOT NULL AND
        additional_content_format_o IS NOT NULL AND
        additional_content_encoding_o IS NOT NULL AND
        additional_content_byte_vo IS NOT NULL
    )
);

-- This index is used so that plum_metadata_seal lookups are fast.
CREATE INDEX plum_metadata_seals ON plum_metadatas(plum_metadata_seal);

CREATE TABLE plum_relations (
    -- Primary key
    plum_relations_rowid INTEGER NOT NULL PRIMARY KEY,

    -- Other DB-oriented attributes.
    row_inserted_at BIGINT NOT NULL,

    -- This is the "real key" for a PlumRelations, i.e. its globally unique ID.
    plum_relations_seal BLOB NOT NULL,

    -- PlumRelations attributes.
    plum_relations_nonce_o BLOB,
    source_plum_body_seal BLOB NOT NULL,

    UNIQUE(plum_relations_seal),
    UNIQUE(source_plum_body_seal)
);

-- This index is used so that plum_relations_seal lookups are fast.
CREATE INDEX plum_relations_seals ON plum_relations(plum_relations_seal);

CREATE TABLE plum_relation_mappings (
    -- Primary key
    plum_relation_mappings_rowid INTEGER NOT NULL PRIMARY KEY,

    -- PlumRelation attributes
    plum_relations_rowid INTEGER NOT NULL,
    target_plum_head_seal BLOB NOT NULL,
    plum_relation_flags INTEGER NOT NULL,

    UNIQUE(plum_relations_rowid, target_plum_head_seal),
    FOREIGN KEY(plum_relations_rowid) REFERENCES plum_relations(plum_relations_rowid)
);

CREATE TABLE plum_bodies (
    -- Primary key
    plum_bodies_rowid INTEGER NOT NULL PRIMARY KEY,

    -- Other DB-oriented attributes.
    row_inserted_at BIGINT,

    -- This is the "real key" for a PlumBody, i.e. its globally unique ID.
    plum_body_seal BLOB NOT NULL,

    -- PlumBody attributes
    plum_body_nonce_o BLOB,
    plum_body_content_length BIGINT NOT NULL,
    plum_body_content_class TEXT NOT NULL,
    plum_body_content_format TEXT NOT NULL,
    plum_body_content_encoding TEXT NOT NULL,
    plum_body_content_byte_v BLOB NOT NULL,

    UNIQUE(plum_body_seal)
);

-- This index is used so that plum_body_seal lookups are fast.
CREATE INDEX plum_body_seals ON plum_bodies(plum_body_seal);

CREATE TABLE path_states (
    -- Primary key
    path_states_rowid INTEGER NOT NULL PRIMARY KEY,

    -- Other DB-oriented attributes.
    row_inserted_at BIGINT NOT NULL,
    row_updated_at BIGINT NOT NULL,
    -- TODO: Use soft deletes so that a non-owner can't resurrect the path and pass themselves off as the original.
    -- row_deleted_at BIGINT,

    -- This is the "real key" for a path state.
    path TEXT NOT NULL,

    -- This is the Plum that this path maps to.
    current_state_plum_head_seal BLOB NOT NULL,

    UNIQUE(path)
);

-- This index is used so that path lookups are fast.
CREATE INDEX paths ON path_states(path);
