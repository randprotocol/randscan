-- Chains 17 and 18 add public fields the explorer stored nowhere:
--
-- * split authorisation (chain 17, fullnode v0.6.3, bundle guest v3 + `hc_auth`): every bundle
--   publishes `auth_commit`, the commitment a second, tiny proof over the spend key must match,
--   and that auth proof rides beside the bundle proof (`tx_json` reports it by length);
-- * the gas section (chain 18, fullnode v0.6.6, constraint set 8): the tip ledger's current gas
--   prices are consensus state that moves per block (`rand_status.gas_prices`), so they sit in the
--   stats row beside `limits` (which keeps the whole `rand_getLimits` reply, the gas section's
--   fixed parameters included);
-- * the auth guest the genesis pins (`rand_status.hc_auth`), beside `hc_bundle`.
--
-- Additive only: no chain table is reshaped and nothing needs re-indexing. Rows indexed before
-- this migration keep the defaults, which are also what a pre-v3 chain reports (a zero commit
-- and an empty auth proof are stored as NULL / 0). Migration 8 stays burned (see 009).

ALTER TABLE transactions
    -- Word8 hex, the bundle proof's `auth_commit`; NULL when the node reported none.
    ADD COLUMN auth_commit    VARCHAR(64),
    -- Bytes of the auth proof; 0 on a chain without an auth guest.
    ADD COLUMN auth_proof_len BIGINT NOT NULL DEFAULT 0;

ALTER TABLE network_stats
    -- The genesis auth guest, hex; NULL on a chain without split authorisation.
    ADD COLUMN hc_auth    VARCHAR(64),
    -- `rand_status.gas_prices` verbatim: { gas_price, byte_price } as decimal strings, the tip's
    -- live prices under a `gas` section; NULL on a chain without one.
    ADD COLUMN gas_prices JSONB;
