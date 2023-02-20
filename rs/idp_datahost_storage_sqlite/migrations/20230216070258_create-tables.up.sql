-- Add up migration script here

-- Drop existing tables, since this is a clean start.

DROP TABLE IF EXISTS plum_bodies;
DROP TABLE IF EXISTS plum_relations;
DROP INDEX IF EXISTS plum_head_body_references;
DROP TABLE IF EXISTS plum_heads;

-- Now create fresh ones.

CREATE TABLE plum_heads (
    -- Primary key
    plum_heads_rowid INTEGER NOT NULL PRIMARY KEY,
    row_inserted_at BIGINT NOT NULL,
    -- This is the "real key" for a PlumHead, i.e. its globally unique ID.
    plum_head_seal BLOB NOT NULL,
    -- PlumHead attributes, using foreign keys to link to the seal values of plum_relations and plum_bodies
    plum_head_nonce_o BLOB,
    plum_relations_seal_o BLOB,
    plum_body_seal BLOB NOT NULL,
    owner_id_o TEXT,
    created_at_o BIGINT,
    metadata_o BLOB,

    UNIQUE(plum_head_seal)
);

-- This index is used so that plum_head_seal lookups are fast.
CREATE INDEX plum_head_seals ON plum_heads(plum_head_seal);

-- This index is used so that plum_head -> plum_body reference counting is efficient.
CREATE INDEX plum_head_body_references ON plum_heads(plum_body_seal);

CREATE TABLE plum_relations (
    -- Primary key
    plum_relations_rowid INTEGER NOT NULL PRIMARY KEY,
    row_inserted_at BIGINT NOT NULL,
    -- This is the "real key" for a PlumRelations, i.e. its globally unique ID.
    plum_relations_seal BLOB NOT NULL,
    -- PlumRelations attributes, using foreign keys to link to the seal values of plum_heads
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
    row_inserted_at BIGINT,
    -- This is the "real key" for a PlumBody, i.e. its globally unique ID.
    plum_body_seal BLOB NOT NULL,
    -- PlumBody attributes
    plum_body_nonce_o BLOB,
    plum_body_content_length BIGINT NOT NULL,
    plum_body_content_type BLOB NOT NULL,
    plum_body_content BLOB NOT NULL,

    UNIQUE(plum_body_seal)
);

-- This index is used so that plum_body_seal lookups are fast.
CREATE INDEX plum_body_seals ON plum_bodies(plum_body_seal);
