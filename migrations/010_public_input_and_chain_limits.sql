-- Chain 14 also carries the v0.4 (chain 13) call limits, which the explorer did not store yet:
-- a program's public input is fixed at deploy (`rand_getProgram`'s `public_words_len` and
-- `public_digest`), every call's proof is checked against it (a receipt's `h_pub`), and the size
-- caps are genesis parameters (`rand_getLimits`) rather than constants. The node's build and the
-- genesis hash (`rand_getVersion`, `rand_getGenesisHash`, v0.3) sit beside them in the stats row.
--
-- Additive only: no chain table is reshaped, so nothing needs re-indexing. Rows indexed before
-- this migration keep the defaults, which are also what the node reports for a program deployed
-- without a public input (length 0, digest null) and a call checked against the empty one
-- (`h_pub` null).

ALTER TABLE programs
    ADD COLUMN public_words_len BIGINT NOT NULL DEFAULT 0,
    -- Word8 hex; NULL for a program deployed without a public input.
    ADD COLUMN public_digest    VARCHAR(64);

ALTER TABLE receipts
    -- Word8 hex, the program's `public_digest`; NULL when the proof was checked against the
    -- digest of the empty public input.
    ADD COLUMN h_pub VARCHAR(64);

ALTER TABLE network_stats
    ADD COLUMN genesis_hash VARCHAR(64),
    ADD COLUMN node_version VARCHAR(32),
    -- The full commit hash, `-dirty` appended by the node's build when the tree was not clean.
    ADD COLUMN node_git_sha VARCHAR(64),
    ADD COLUMN fri_profile  VARCHAR(32),
    -- `rand_getLimits` verbatim: max_program_words, max_proof_bytes, max_block_bytes,
    -- max_call_envelope_bytes, max_program_public_words. NULL on a node without the method.
    ADD COLUMN limits       JSONB;
