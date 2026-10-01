-- Chain 20 (fullnode v0.6.8, 2026-10-01): zUSD bridge fees and audit v6's actions.
--
-- `bridge.fees` (fullnode docs/bridge.md §25): a deposit locks its gross and mints two notes —
-- the depositor's `deposit_amount` (net) and a fee note to the genesis fee recipient; a burn
-- destroys `amount`, releases `release_amount` (what leaves `locked`) and mints the fee back as a
-- note. A fee note is an extra leaf in the commitment tree, with no envelope; `fee_cm` links that
-- leaf to its transaction the way `derived_cm` and `payout_cms` do.
--
-- Audit v6: `admit_validator` and `slash_equivocation` are kept whole in `staking_action`; the
-- rotations and `cancel_rotation` go in `bridge_governance` like the other guardian actions.
--
-- The chain-20 rows indexed before this migration carry none of these fields, so the chain data
-- is dropped and re-indexed from height 0 under the same chain id (users, sessions and API keys
-- are not chain data and are kept; the node holds the whole chain, ~2 700 blocks at the time).

ALTER TABLE transactions
    ADD COLUMN deposit_amount  NUMERIC,
    ADD COLUMN release_amount  NUMERIC,
    ADD COLUMN fee_note        JSONB,
    ADD COLUMN fee_cm          VARCHAR(64),
    ADD COLUMN staking_action  JSONB;

CREATE INDEX idx_tx_fee_cm ON transactions (fee_cm) WHERE fee_cm IS NOT NULL;

TRUNCATE nullifiers, notes, receipts, programs, transactions, blocks, validators RESTART IDENTITY CASCADE;

UPDATE indexer_state SET next_height = 0, next_leaf = 0, last_indexed_hash = NULL, is_syncing = FALSE,
    updated_at = NOW() WHERE id = 1;
