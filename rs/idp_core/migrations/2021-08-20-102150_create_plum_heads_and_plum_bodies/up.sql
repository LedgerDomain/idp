-- Your SQL goes here

CREATE TABLE plum_heads (
    -- Primary key
    head_seal BLOB NOT NULL PRIMARY KEY,
    -- DB-only values
    row_inserted_at BIGINT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- PlumHead attributes
    body_seal BLOB NOT NULL,
    body_length BIGINT NOT NULL,
    body_content_type BLOB NOT NULL,
    head_nonce_o BLOB,
    owner_did_o TEXT,
    created_at_o BIGINT,
    metadata_o BLOB,
    relations_seal_o BLOB
);

-- This index is used so that plum_head -> plum_body reference counting is efficient.
CREATE INDEX plum_head_body_references ON plum_heads(body_seal);

CREATE TABLE plum_relations (
    -- DB-only values
    row_inserted_at BIGINT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- PlumRelation attributes
    source_head_seal BLOB NOT NULL,
    target_head_seal BLOB NOT NULL,
    relation_flags INTEGER NOT NULL,
    PRIMARY KEY(source_head_seal, target_head_seal)
);

CREATE TABLE plum_bodies (
    -- Primary key
    body_seal BLOB NOT NULL PRIMARY KEY,
    -- DB-only values
    row_inserted_at BIGINT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- PlumBody attributes
    body_nonce_o BLOB,
    body_content_o BLOB
);
