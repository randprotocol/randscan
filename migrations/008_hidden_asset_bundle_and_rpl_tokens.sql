-- Chain 14: the hidden-asset bundle (4 inputs / 4 outputs, the asset private), the RPL token
-- standard (register_token, token_mint, set_authority, token_burn) and bridge hardening's
-- governance actions (pause_mints, unpause_mints, register_bridged_token, list_backing) plus
-- bridge_attest's PQ co-signers. A hard fork like every chain cut before it (constraint sets,
-- the RAND rename, the short-address revert): the wire format changes, so old rows do not carry
-- over. This migration only reshapes the schema; `BlockProcessor::reset_chain` (indexer/service)
-- drops and re-indexes the chain tables from height 0 when the indexer's `chain_id` changes.
--
-- The bundle no longer has a public `asset` field at all: slots 0-1 carry a private asset (RAND
-- or any RPL token) and slots 2-3 always RAND, so there is no column for it, and there must never
-- be one added back — a transfer's asset is not something this database is allowed to know.

ALTER TABLE transactions
    ADD COLUMN nullifier_3    VARCHAR(64),
    ADD COLUMN nullifier_4    VARCHAR(64),
    ADD COLUMN commitment_3   VARCHAR(64),
    ADD COLUMN commitment_4   VARCHAR(64),
    ADD COLUMN envelope_len_3 BIGINT,
    ADD COLUMN envelope_len_4 BIGINT,
    -- Replaces `burn`/`asset` (the old two-bundle shape's single burn+asset pair).
    ADD COLUMN burn_a         NUMERIC(40,0),
    ADD COLUMN burn_r         NUMERIC(40,0),
    ADD COLUMN burn_asset     BIGINT,
    -- bridge_burn: the backing being redeemed (a source-chain token address, 32 bytes hex).
    ADD COLUMN bridge_token   VARCHAR(64),
    -- bridge_attest / token_mint / register_token (initial mint): the chain-computed note's
    -- blinding (public, a field of the signed action) and its commitment. `derived_cm` is set by
    -- the indexer itself for token_mint/register_token (the node does not publish their
    -- commitment directly, unlike a bridge_attest's, which is copied here from `tx_json`'s
    -- `commitment` field) — see `randscan_core::notecommit::mint_commitment`. It is what links
    -- that tree leaf (a real entry `rand_getCommitments` serves an envelope for, same as any
    -- other note) back to this transaction, the same way `commitment_1..4` already do for a
    -- bundle's own outputs.
    ADD COLUMN deposit_r      VARCHAR(64),
    ADD COLUMN derived_cm     VARCHAR(64),
    -- bridge_attest / unpause_mints / register_bridged_token / list_backing: the PQ (Dilithium2)
    -- guardians who co-signed, by index.
    ADD COLUMN pq_signers     INTEGER[],
    -- register_token / token_mint / set_authority / token_burn: the action's own fields in full
    -- (name, symbol, decimals, authority, index, initial mint, nonce, new authority, ...) — the
    -- `asset_bundle` JSONB precedent, one action over. `asset_index`/`amount` (existing columns)
    -- still carry the one number list views and the supply-history query need, so they need no
    -- JSONB reach-through.
    ADD COLUMN token_action      JSONB,
    -- pause_mints / unpause_mints / register_bridged_token / list_backing: likewise in full.
    ADD COLUMN bridge_governance JSONB;

UPDATE transactions SET burn_a = burn, burn_asset = asset WHERE burn IS NOT NULL OR asset IS NOT NULL;

ALTER TABLE transactions
    DROP COLUMN burn,
    DROP COLUMN asset,
    DROP COLUMN asset_bundle;

CREATE INDEX idx_tx_commitment_3 ON transactions(commitment_3) WHERE commitment_3 IS NOT NULL;
CREATE INDEX idx_tx_commitment_4 ON transactions(commitment_4) WHERE commitment_4 IS NOT NULL;
CREATE INDEX idx_tx_derived_cm ON transactions(derived_cm) WHERE derived_cm IS NOT NULL;
-- The supply-history query for a token page: every register_token/token_mint/token_burn that
-- named this registry index, oldest first.
CREATE INDEX idx_tx_asset_index_kind ON transactions(asset_index, kind, height) WHERE asset_index IS NOT NULL;

-- Chain 14 is a new chain; re-indexing from 0 rebuilds these, but the reset itself only truncates
-- the tables migration 005 created, so this chain cut needs nothing new dropped or added there.
