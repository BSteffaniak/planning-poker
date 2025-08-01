-- Revert votes.id from BIGSERIAL (INT8) back to SERIAL (INT4)
-- WARNING: This will fail if any ID values exceed INT4 range (2,147,483,647)

ALTER TABLE votes ALTER COLUMN id TYPE INTEGER;
ALTER SEQUENCE votes_id_seq AS INTEGER;
