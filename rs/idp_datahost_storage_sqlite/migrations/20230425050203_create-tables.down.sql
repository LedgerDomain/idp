-- Tear everything down in reverse order.

DROP INDEX IF EXISTS paths;
DROP TABLE IF EXISTS path_states;

DROP INDEX IF EXISTS plum_body_seals;
DROP TABLE IF EXISTS plum_bodies;
DROP TABLE IF EXISTS plum_relation_mappings;
DROP INDEX IF EXISTS plum_relations_seals;
DROP TABLE IF EXISTS plum_relations;
DROP INDEX IF EXISTS plum_metadata_seals;
DROP TABLE IF EXISTS plum_metadatas;
DROP INDEX IF EXISTS plum_head_body_references;
DROP INDEX IF EXISTS plum_head_relations_references;
DROP INDEX IF EXISTS plum_head_metadata_references;
DROP INDEX IF EXISTS plum_head_seals;
DROP TABLE IF EXISTS plum_heads;
