-- RandScan schema for the Rand Protocol SHRUGG chain (shrugg-node, JSON-RPC shrugg_*).
-- Amounts (units of SHRUGG, u128) are NUMERIC(40,0); the API serves them as decimal strings.

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
    hash          VARCHAR(64) PRIMARY KEY,
    block_hash    VARCHAR(64) NOT NULL REFERENCES blocks(hash) ON DELETE CASCADE,
    height        BIGINT NOT NULL,
    tx_index      INTEGER NOT NULL,
    sender        VARCHAR(64) NOT NULL,
    nonce         BIGINT NOT NULL,
    fee           NUMERIC(40,0) NOT NULL DEFAULT 0,
    kind          VARCHAR(16) NOT NULL,           -- transfer | mint | deploy | call
    chain_id      BIGINT NOT NULL,
    timestamp_ms  BIGINT NOT NULL,
    -- transfer / mint
    to_address    VARCHAR(64),
    amount        NUMERIC(40,0),
    -- deploy / call
    program_id    VARCHAR(64),
    base_pc       BIGINT,
    words_len     BIGINT,
    proof_len     BIGINT,
    recipients    TEXT[] NOT NULL DEFAULT '{}',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_tx_height_desc ON transactions(height DESC, tx_index DESC);
CREATE INDEX idx_tx_sender ON transactions(sender, height DESC);
CREATE INDEX idx_tx_kind ON transactions(kind, height DESC);
CREATE INDEX idx_tx_program ON transactions(program_id, height DESC) WHERE program_id IS NOT NULL;
CREATE INDEX idx_tx_to ON transactions(to_address) WHERE to_address IS NOT NULL;

CREATE TABLE receipts (
    tx_hash        VARCHAR(64) PRIMARY KEY REFERENCES transactions(hash) ON DELETE CASCADE,
    program        VARCHAR(64) NOT NULL,
    tier           INTEGER NOT NULL,
    outputs        BIGINT[] NOT NULL DEFAULT '{}',
    effect_to      VARCHAR(64),
    effect_amount  NUMERIC(40,0),
    height         BIGINT NOT NULL,
    tx_index       INTEGER NOT NULL
);

CREATE TABLE accounts (
    address            VARCHAR(64) PRIMARY KEY,
    balance            NUMERIC(40,0) NOT NULL DEFAULT 0,
    nonce              BIGINT NOT NULL DEFAULT 0,
    tx_count           BIGINT NOT NULL DEFAULT 0,
    first_seen_height  BIGINT NOT NULL DEFAULT 0,
    last_seen_height   BIGINT NOT NULL DEFAULT 0,
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_accounts_balance ON accounts(balance DESC);

CREATE TABLE account_transactions (
    id        BIGSERIAL PRIMARY KEY,
    account   VARCHAR(64) NOT NULL,
    tx_hash   VARCHAR(64) NOT NULL REFERENCES transactions(hash) ON DELETE CASCADE,
    role      VARCHAR(16) NOT NULL,                -- sender | recipient
    height    BIGINT NOT NULL,
    tx_index  INTEGER NOT NULL,
    UNIQUE(account, tx_hash, role)
);

CREATE INDEX idx_account_tx ON account_transactions(account, height DESC, tx_index DESC);

CREATE TABLE validators (
    address     VARCHAR(64) PRIMARY KEY,
    stake       NUMERIC(40,0) NOT NULL DEFAULT 0,
    sort_index  INTEGER NOT NULL DEFAULT 0,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE programs (
    id                  VARCHAR(64) PRIMARY KEY,
    deployer            VARCHAR(64) NOT NULL,
    deploy_tx           VARCHAR(64) NOT NULL REFERENCES transactions(hash) ON DELETE CASCADE,
    deployed_at_height  BIGINT NOT NULL,
    base_pc             BIGINT NOT NULL DEFAULT 0,
    words_len           BIGINT NOT NULL DEFAULT 0,
    code_hash           VARCHAR(64) NOT NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_programs_deployer ON programs(deployer);
CREATE INDEX idx_programs_height ON programs(deployed_at_height DESC);

CREATE TABLE network_stats (
    id                  INTEGER PRIMARY KEY DEFAULT 1 CHECK (id = 1),
    chain_id            BIGINT NOT NULL DEFAULT 0,
    symbol              VARCHAR(16) NOT NULL DEFAULT 'SHRUGG',
    decimals            SMALLINT NOT NULL DEFAULT 9,
    height              BIGINT NOT NULL DEFAULT 0,
    view                BIGINT NOT NULL DEFAULT 0,
    total_transactions  BIGINT NOT NULL DEFAULT 0,
    total_accounts      BIGINT NOT NULL DEFAULT 0,
    validator_count     BIGINT NOT NULL DEFAULT 0,
    total_stake         NUMERIC(40,0) NOT NULL DEFAULT 0,
    total_supply        NUMERIC(40,0) NOT NULL DEFAULT 0,
    program_count       BIGINT NOT NULL DEFAULT 0,
    avg_block_time_ms   DOUBLE PRECISION NOT NULL DEFAULT 0,
    peer_count          INTEGER NOT NULL DEFAULT 0,
    mempool_size        INTEGER NOT NULL DEFAULT 0,
    node_syncing        BOOLEAN NOT NULL DEFAULT FALSE,
    faucet              BOOLEAN NOT NULL DEFAULT FALSE,
    confidential        BOOLEAN NOT NULL DEFAULT FALSE,
    current_leader      VARCHAR(64),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO network_stats (id) VALUES (1);

CREATE TABLE indexer_state (
    id                  INTEGER PRIMARY KEY DEFAULT 1 CHECK (id = 1),
    next_height         BIGINT NOT NULL DEFAULT 0,
    last_indexed_hash   VARCHAR(64),
    is_syncing          BOOLEAN NOT NULL DEFAULT FALSE,
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO indexer_state (id) VALUES (1);

-- Geolocation cache for peer IPs (nodes map)
CREATE TABLE node_geo (
    ip            VARCHAR(64) PRIMARY KEY,
    lat           DOUBLE PRECISION,
    lon           DOUBLE PRECISION,
    city          VARCHAR(128),
    region        VARCHAR(128),
    country       VARCHAR(128),
    country_code  VARCHAR(8),
    org           VARCHAR(256),
    ok            BOOLEAN NOT NULL DEFAULT FALSE,
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
