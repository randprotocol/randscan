-- Bridge transaction kinds (bridge_attest, bridge_burn) from the fullnode bridge merge, and the
-- chain id the indexed data belongs to (so a chain switch re-indexes without dropping accounts).

ALTER TABLE transactions ALTER COLUMN kind TYPE VARCHAR(32);   -- also holds unknown node tags verbatim

ALTER TABLE transactions
    ADD COLUMN asset         VARCHAR(64),      -- bridge_burn: asset id (hex)
    ADD COLUMN bridge_amount NUMERIC(40,0),    -- bridge_burn: bridged units (8 decimals)
    ADD COLUMN to_chain      INTEGER,          -- bridge_burn: destination chain id
    ADD COLUMN bridge_to     VARCHAR(64),      -- bridge_burn: destination address, 32 bytes hex
    ADD COLUMN bridge_fee    NUMERIC(40,0),    -- bridge_burn: relayer fee, bridged units
    ADD COLUMN attestation   TEXT;             -- bridge_attest: signed message, hex (up to 32 KiB)

ALTER TABLE indexer_state ADD COLUMN chain_id BIGINT;   -- NULL until the indexer first talks to a node
