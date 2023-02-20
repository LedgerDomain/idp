-- Add down migration script here

DROP INDEX IF EXISTS plum_body_seals;
DROP TABLE IF EXISTS plum_bodies;
DROP TABLE IF EXISTS plum_relation_mappings;
DROP INDEX IF EXISTS plum_relations_seals;
DROP TABLE IF EXISTS plum_relations;
DROP INDEX IF EXISTS plum_head_body_references;
DROP INDEX IF EXISTS plum_head_seals;
DROP TABLE IF EXISTS plum_heads;
