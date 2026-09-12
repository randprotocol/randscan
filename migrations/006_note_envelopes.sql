-- Store each tree leaf's envelope (kem_ct, to_receiver, to_sender, body; hex) so the explorer can
-- hand it to the browser, where a viewing key or a per-transaction key opens it locally. The
-- node serves envelopes to everyone (shrugg_getCommitments); nothing here is secret. Leaves are
-- re-fetched from 0 to backfill envelopes for notes indexed before this column existed.

ALTER TABLE notes ADD COLUMN envelope JSONB;

UPDATE indexer_state SET next_leaf = 0, updated_at = NOW() WHERE id = 1;
