-- RPL-2 (fullnode v0.6.8, genesis-gated on a `program_state` section; on no chain as of
-- 2026-10-01): the `invoke` action, a call whose proof vouches for one declared state transition
-- of the program, which the ledger applies — cells read and written, value into the program's
-- vault (the bundle's burn fields), and up to four chain-computed notes out of it (`pays`, from
-- the vault; `mints`, new units of a token the program is the authority of).
--
-- The transition is kept whole (every field is public by design, spec §2/§9) and the payout
-- notes' commitments beside it, so a leaf the chain appended for a payout links back to the
-- invoke the way a `token_mint`'s derived note does (`derived_cm`). Additive only: nothing is
-- reshaped and nothing re-indexed; an invoke indexed by an older build as kind `invoke` with
-- these columns NULL would need a re-index, but no chain has carried one.

ALTER TABLE transactions
    -- invoke: the transition as the node renders it (reads, writes, inflow, pays, mints).
    ADD COLUMN transition  JSONB,
    -- invoke: the commitments of its payout notes, pays then mints, in tree order.
    ADD COLUMN payout_cms  VARCHAR(64)[];

CREATE INDEX idx_tx_payout_cms ON transactions USING GIN (payout_cms) WHERE payout_cms IS NOT NULL;
