-- The native coin is RAND; gas is paid in RAND. The indexer writes the symbol on every stats
-- refresh, so only the column default needs to follow the rename.
ALTER TABLE network_stats ALTER COLUMN symbol SET DEFAULT 'RAND';
