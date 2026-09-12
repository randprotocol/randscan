-- The shielded chain (fullnode shielded-pool phases S1-S3): no accounts, every transaction is a
-- bundle plus an action, the commitment tree and the nullifier set are public, the validator
-- register carries stake/rewards/pending in the clear. The chain tables are rebuilt from scratch
-- (the shielded chain is a new chain; account-chain data cannot be carried over). User accounts,
-- sessions, API keys, password resets and the peer geolocation cache are untouched.

DROP TABLE IF EXISTS account_transactions;
DROP TABLE IF EXISTS receipts;
DROP TABLE IF EXISTS programs;
DROP TABLE IF EXISTS transactions;
DROP TABLE IF EXISTS accounts;
DROP TABLE IF EXISTS validators;
DROP TABLE IF EXISTS blocks;

CREATE TABLE blocks (
    hash          VARCHAR(64) PRIMARY KEY,
    height        BIGINT NOT NULL UNIQUE,
    view          BIGINT NOT NULL,
    parent        VARCHAR(64) NOT NULL,
    proposer      VARCHAR(64) NOT NULL,
    timestamp_ms  BIGINT NOT NULL,
    tx_root       VARCHAR(64) NOT NULL,
    state_root    VARCHAR(64) NOT NULL,
    justify_view  BIGINT NOT NULL,
    tx_count      INTEGER NOT NULL DEFAULT 0,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_blocks_height_desc ON blocks(height DESC);
CREATE INDEX idx_blocks_proposer ON blocks(proposer, height DESC);

CREATE TABLE transactions (
    hash                VARCHAR(64) PRIMARY KEY,
    block_hash          VARCHAR(64) NOT NULL REFERENCES blocks(hash) ON DELETE CASCADE,
    height              BIGINT NOT NULL,
    tx_index            INTEGER NOT NULL,
    chain_id            BIGINT NOT NULL,
    timestamp_ms        BIGINT NOT NULL,
    -- transfer | mint | deploy | call | bond | unbond | withdraw | bridge_attest | bridge_burn,
    -- or the node's own tag for a kind this build does not decode (served as `other`)
    kind                VARCHAR(32) NOT NULL,
    fee                 NUMERIC(40,0) NOT NULL DEFAULT 0,
    -- the fee bundle (null columns when the action is validator-signed and carries none)
    has_bundle          BOOLEAN NOT NULL DEFAULT FALSE,
    anchor              VARCHAR(64),
    nullifier_1         VARCHAR(64),
    nullifier_2         VARCHAR(64),
    commitment_1        VARCHAR(64),
    commitment_2        VARCHAR(64),
    burn                NUMERIC(40,0),
    asset               BIGINT,
    bundle_time         BIGINT,
    proof_len           BIGINT,
    envelope_len_1      BIGINT,
    envelope_len_2      BIGINT,
    -- deploy / call
    program_id          VARCHAR(64),
    words_len           BIGINT,
    call_proof_len      BIGINT,
    input_envelope_len  BIGINT,
    -- mint / bond / unbond / withdraw / bridge_attest / bridge_burn: the one public amount
    amount              NUMERIC(40,0),
    -- mint
    cm                  VARCHAR(64),
    -- mint (minter) / bond / unbond / withdraw
    validator           VARCHAR(64),
    registered          BOOLEAN,
    action_nonce        BIGINT,
    -- bridge_attest
    attestation_len     BIGINT,
    recipient           TEXT,
    note_time           BIGINT,
    -- bridge_attest / bridge_burn: the bridged asset's registry index
    asset_index         BIGINT,
    -- bridge_burn
    relayer_fee         NUMERIC(40,0),
    to_chain            INTEGER,
    bridge_to           VARCHAR(64),
    asset_bundle        JSONB,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_tx_height_desc ON transactions(height DESC, tx_index DESC);
CREATE INDEX idx_tx_kind ON transactions(kind, height DESC);
CREATE INDEX idx_tx_program ON transactions(program_id, height DESC) WHERE program_id IS NOT NULL;
CREATE INDEX idx_tx_validator ON transactions(validator, height DESC) WHERE validator IS NOT NULL;
CREATE INDEX idx_tx_commitment_1 ON transactions(commitment_1) WHERE commitment_1 IS NOT NULL;
CREATE INDEX idx_tx_commitment_2 ON transactions(commitment_2) WHERE commitment_2 IS NOT NULL;
CREATE INDEX idx_tx_cm ON transactions(cm) WHERE cm IS NOT NULL;

-- Every nullifier a bundle published (both of the fee bundle, both of a burn's asset bundle).
CREATE TABLE nullifiers (
    nullifier  VARCHAR(64) PRIMARY KEY,
    tx_hash    VARCHAR(64) NOT NULL REFERENCES transactions(hash) ON DELETE CASCADE,
    height     BIGINT NOT NULL,
    tx_index   INTEGER NOT NULL
);

CREATE INDEX idx_nullifiers_height ON nullifiers(height DESC);

-- The commitment tree, leaf by leaf, from shrugg_getCommitments. tx_hash is set when the
-- commitment was on the wire of an indexed transaction (bundle outputs, mints).
CREATE TABLE notes (
    leaf_index  BIGINT PRIMARY KEY,
    cm          VARCHAR(64) NOT NULL UNIQUE,
    height      BIGINT NOT NULL,
    tx_hash     VARCHAR(64)
);

CREATE INDEX idx_notes_height ON notes(height DESC, leaf_index DESC);

CREATE TABLE receipts (
    tx_hash   VARCHAR(64) PRIMARY KEY REFERENCES transactions(hash) ON DELETE CASCADE,
    program   VARCHAR(64) NOT NULL,
    tier      INTEGER NOT NULL,
    outputs   BIGINT[] NOT NULL DEFAULT '{}',
    height    BIGINT NOT NULL,
    tx_index  INTEGER NOT NULL,
    h_in      VARCHAR(64) NOT NULL DEFAULT ''
);

-- The validator register (spec §8). `pending` is [{release_epoch, amount}] oldest first.
CREATE TABLE validators (
    address     VARCHAR(64) PRIMARY KEY,
    stake       NUMERIC(40,0) NOT NULL DEFAULT 0,
    rewards     NUMERIC(40,0) NOT NULL DEFAULT 0,
    pending     JSONB NOT NULL DEFAULT '[]',
    payout      TEXT,
    nonce       BIGINT NOT NULL DEFAULT 0,
    active      BOOLEAN NOT NULL DEFAULT TRUE,
    sort_index  INTEGER NOT NULL DEFAULT 0,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE programs (
    id                  VARCHAR(64) PRIMARY KEY,
    deploy_tx           VARCHAR(64) NOT NULL REFERENCES transactions(hash) ON DELETE CASCADE,
    deployed_at_height  BIGINT NOT NULL,
    base_pc             BIGINT NOT NULL DEFAULT 0,
    words_len           BIGINT NOT NULL DEFAULT 0,
    code_hash           VARCHAR(64) NOT NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_programs_height ON programs(deployed_at_height DESC);

ALTER TABLE network_stats
    DROP COLUMN total_accounts,
    ADD COLUMN notes                  BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN nullifiers             BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN active_validator_count BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN pool_value             NUMERIC(40,0),
    ADD COLUMN tree_root              VARCHAR(64),
    ADD COLUMN hc_bundle              VARCHAR(64),
    ADD COLUMN epoch                  BIGINT,
    ADD COLUMN epoch_blocks           BIGINT;

UPDATE network_stats SET height = 0, view = 0, total_transactions = 0, validator_count = 0,
    total_stake = 0, total_supply = 0, program_count = 0, avg_block_time_ms = 0,
    current_leader = NULL, updated_at = NOW() WHERE id = 1;

ALTER TABLE indexer_state ADD COLUMN next_leaf BIGINT NOT NULL DEFAULT 0;

UPDATE indexer_state SET next_height = 0, last_indexed_hash = NULL, is_syncing = FALSE,
    chain_id = NULL, updated_at = NOW() WHERE id = 1;
