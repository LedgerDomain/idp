-- Add up migration script here

CREATE TABLE path_states (
    -- Primary key
    path_states_rowid INTEGER NOT NULL PRIMARY KEY,
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
