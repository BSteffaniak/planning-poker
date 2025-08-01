-- Change votes.id from SERIAL (INT4) to BIGSERIAL (INT8)
-- This expands the primary key to support more votes and matches Rust i64 expectations

ALTER TABLE votes ALTER COLUMN id TYPE BIGINT;
ALTER SEQUENCE votes_id_seq AS BIGINT;
