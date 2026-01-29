-- RandScan Initial Database Schema
-- PostgreSQL migrations for RandProtocol blockchain explorer

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ============================================================================
-- BLOCKS AND CONSENSUS
-- ============================================================================

-- Blocks table
CREATE TABLE blocks (
    block_id VARCHAR(64) PRIMARY KEY,
    height BIGINT NOT NULL UNIQUE,
    view_number BIGINT NOT NULL,
    epoch BIGINT NOT NULL,
    parent_id VARCHAR(64) NOT NULL,
    proposer_id VARCHAR(64) NOT NULL,
    transactions_root VARCHAR(64) NOT NULL,
    state_root VARCHAR(64) NOT NULL,
    supply_commitment VARCHAR(64) NOT NULL,
    timestamp BIGINT NOT NULL,
    transaction_count INTEGER NOT NULL DEFAULT 0,
    finalized BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_blocks_height ON blocks(height DESC);
CREATE INDEX idx_blocks_epoch ON blocks(epoch);
CREATE INDEX idx_blocks_proposer ON blocks(proposer_id);
CREATE INDEX idx_blocks_timestamp ON blocks(timestamp DESC);
CREATE INDEX idx_blocks_finalized ON blocks(finalized) WHERE NOT finalized;

-- Quorum Certificates table
CREATE TABLE quorum_certificates (
    block_id VARCHAR(64) PRIMARY KEY REFERENCES blocks(block_id) ON DELETE CASCADE,
    vote_type VARCHAR(20) NOT NULL,
    view_number BIGINT NOT NULL,
    certified_block_id VARCHAR(64) NOT NULL,
    certified_block_height BIGINT NOT NULL,
    signer_count INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_qc_view ON quorum_certificates(view_number);

-- QC Signers table (many-to-many relationship)
CREATE TABLE qc_signers (
    id SERIAL PRIMARY KEY,
    block_id VARCHAR(64) NOT NULL REFERENCES blocks(block_id) ON DELETE CASCADE,
    validator_id VARCHAR(64) NOT NULL,
    signature VARCHAR(128) NOT NULL,
    UNIQUE(block_id, validator_id)
);

CREATE INDEX idx_qc_signers_block ON qc_signers(block_id);
CREATE INDEX idx_qc_signers_validator ON qc_signers(validator_id);

-- ============================================================================
-- TRANSACTIONS (9 types)
-- ============================================================================

-- Main transactions table
CREATE TABLE transactions (
    tx_id VARCHAR(64) PRIMARY KEY,
    block_id VARCHAR(64) REFERENCES blocks(block_id) ON DELETE SET NULL,
    block_height BIGINT,
    sender VARCHAR(64) NOT NULL,
    nonce BIGINT NOT NULL,
    compute_budget BIGINT NOT NULL DEFAULT 0,
    fee BIGINT NOT NULL,
    payload_type VARCHAR(30) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    timestamp BIGINT NOT NULL,
    signature VARCHAR(128) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_tx_block ON transactions(block_id);
CREATE INDEX idx_tx_sender ON transactions(sender);
CREATE INDEX idx_tx_type ON transactions(payload_type);
CREATE INDEX idx_tx_status ON transactions(status);
CREATE INDEX idx_tx_timestamp ON transactions(timestamp DESC);
CREATE INDEX idx_tx_block_height ON transactions(block_height DESC);

-- Public transactions (SVM instructions)
CREATE TABLE tx_public (
    tx_id VARCHAR(64) PRIMARY KEY REFERENCES transactions(tx_id) ON DELETE CASCADE,
    instructions BYTEA NOT NULL,
    instructions_size INTEGER NOT NULL
);

-- Private transactions (encrypted with ZK proof)
CREATE TABLE tx_private (
    tx_id VARCHAR(64) PRIMARY KEY REFERENCES transactions(tx_id) ON DELETE CASCADE,
    encrypted_payload BYTEA NOT NULL,
    proof BYTEA NOT NULL,
    encrypted_payload_size INTEGER NOT NULL,
    proof_size INTEGER NOT NULL
);

-- Stealth transactions
CREATE TABLE tx_stealth (
    tx_id VARCHAR(64) PRIMARY KEY REFERENCES transactions(tx_id) ON DELETE CASCADE,
    ephemeral_pubkey VARCHAR(64) NOT NULL,
    stealth_address VARCHAR(64) NOT NULL,
    encrypted_amount VARCHAR(96) NOT NULL,
    proof BYTEA NOT NULL,
    proof_size INTEGER NOT NULL
);

-- Stake transactions
CREATE TABLE tx_stake (
    tx_id VARCHAR(64) PRIMARY KEY REFERENCES transactions(tx_id) ON DELETE CASCADE,
    amount BIGINT NOT NULL
);

-- Unstake transactions
CREATE TABLE tx_unstake (
    tx_id VARCHAR(64) PRIMARY KEY REFERENCES transactions(tx_id) ON DELETE CASCADE,
    amount BIGINT NOT NULL
);

-- Transfer transactions
CREATE TABLE tx_transfer (
    tx_id VARCHAR(64) PRIMARY KEY REFERENCES transactions(tx_id) ON DELETE CASCADE,
    recipient VARCHAR(64) NOT NULL,
    amount BIGINT NOT NULL
);

CREATE INDEX idx_tx_transfer_recipient ON tx_transfer(recipient);

-- Deploy transactions
CREATE TABLE tx_deploy (
    tx_id VARCHAR(64) PRIMARY KEY REFERENCES transactions(tx_id) ON DELETE CASCADE,
    code BYTEA NOT NULL,
    code_size INTEGER NOT NULL,
    program_id VARCHAR(64)
);

-- Invoke transactions
CREATE TABLE tx_invoke (
    tx_id VARCHAR(64) PRIMARY KEY REFERENCES transactions(tx_id) ON DELETE CASCADE,
    program_id VARCHAR(64) NOT NULL,
    instruction BYTEA NOT NULL,
    instruction_size INTEGER NOT NULL
);

CREATE INDEX idx_tx_invoke_program ON tx_invoke(program_id);

-- Private transfer transactions
CREATE TABLE tx_private_transfer (
    tx_id VARCHAR(64) PRIMARY KEY REFERENCES transactions(tx_id) ON DELETE CASCADE,
    proof BYTEA NOT NULL,
    proof_size INTEGER NOT NULL
);

-- ============================================================================
-- ACCOUNTS
-- ============================================================================

-- Accounts table (dual token: ATLAS + SHRUG)
CREATE TABLE accounts (
    address VARCHAR(64) PRIMARY KEY,
    atlas_balance BIGINT NOT NULL DEFAULT 0,
    shrug_balance BIGINT NOT NULL DEFAULT 0,
    nonce BIGINT NOT NULL DEFAULT 0,
    is_executable BOOLEAN NOT NULL DEFAULT FALSE,
    owner VARCHAR(64),
    data_len INTEGER NOT NULL DEFAULT 0,
    tx_count INTEGER NOT NULL DEFAULT 0,
    first_seen BIGINT NOT NULL,
    last_seen BIGINT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_accounts_atlas ON accounts(atlas_balance DESC);
CREATE INDEX idx_accounts_shrug ON accounts(shrug_balance DESC);
CREATE INDEX idx_accounts_executable ON accounts(is_executable) WHERE is_executable;
CREATE INDEX idx_accounts_tx_count ON accounts(tx_count DESC);

-- Account-Transaction relationship
CREATE TABLE account_transactions (
    id SERIAL PRIMARY KEY,
    account VARCHAR(64) NOT NULL REFERENCES accounts(address) ON DELETE CASCADE,
    tx_id VARCHAR(64) NOT NULL REFERENCES transactions(tx_id) ON DELETE CASCADE,
    role VARCHAR(20) NOT NULL, -- sender, receiver, signer, program
    block_height BIGINT NOT NULL,
    timestamp BIGINT NOT NULL,
    UNIQUE(account, tx_id, role)
);

CREATE INDEX idx_account_tx_account ON account_transactions(account);
CREATE INDEX idx_account_tx_timestamp ON account_transactions(account, timestamp DESC);

-- ============================================================================
-- VALIDATORS
-- ============================================================================

-- Validators table
CREATE TABLE validators (
    validator_id VARCHAR(64) PRIMARY KEY,
    pubkey VARCHAR(64) NOT NULL,
    stake BIGINT NOT NULL DEFAULT 0,
    commission_rate SMALLINT NOT NULL DEFAULT 10,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    blocks_produced BIGINT NOT NULL DEFAULT 0,
    blocks_skipped BIGINT NOT NULL DEFAULT 0,
    last_vote_height BIGINT,
    uptime_percentage DOUBLE PRECISION NOT NULL DEFAULT 100.0,
    first_seen BIGINT NOT NULL,
    last_seen BIGINT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_validators_stake ON validators(stake DESC);
CREATE INDEX idx_validators_active ON validators(is_active) WHERE is_active;
CREATE INDEX idx_validators_uptime ON validators(uptime_percentage DESC);

-- Validator stake history
CREATE TABLE validator_stake_history (
    id SERIAL PRIMARY KEY,
    validator_id VARCHAR(64) NOT NULL REFERENCES validators(validator_id) ON DELETE CASCADE,
    epoch BIGINT NOT NULL,
    stake BIGINT NOT NULL,
    delegators INTEGER NOT NULL DEFAULT 0,
    rewards BIGINT NOT NULL DEFAULT 0,
    timestamp BIGINT NOT NULL,
    UNIQUE(validator_id, epoch)
);

CREATE INDEX idx_stake_history_validator ON validator_stake_history(validator_id);
CREATE INDEX idx_stake_history_epoch ON validator_stake_history(epoch);

-- ============================================================================
-- EPOCHS
-- ============================================================================

CREATE TABLE epochs (
    epoch BIGINT PRIMARY KEY,
    start_height BIGINT NOT NULL,
    end_height BIGINT,
    start_timestamp BIGINT NOT NULL,
    end_timestamp BIGINT,
    block_count INTEGER NOT NULL DEFAULT 0,
    tx_count INTEGER NOT NULL DEFAULT 0,
    validator_count INTEGER NOT NULL DEFAULT 0,
    total_stake BIGINT NOT NULL DEFAULT 0,
    total_rewards BIGINT NOT NULL DEFAULT 0,
    is_current BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_epochs_current ON epochs(is_current) WHERE is_current;

-- ============================================================================
-- TOKENS (SPL tokens beyond ATLAS/SHRUG)
-- ============================================================================

-- Token mints
CREATE TABLE token_mints (
    mint_address VARCHAR(64) PRIMARY KEY,
    symbol VARCHAR(20) NOT NULL,
    name VARCHAR(100) NOT NULL,
    decimals SMALLINT NOT NULL DEFAULT 9,
    total_supply BIGINT NOT NULL DEFAULT 0,
    circulating_supply BIGINT NOT NULL DEFAULT 0,
    burned BIGINT NOT NULL DEFAULT 0,
    holder_count INTEGER NOT NULL DEFAULT 0,
    tx_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_token_mints_symbol ON token_mints(symbol);

-- Token accounts (balances)
CREATE TABLE token_accounts (
    id SERIAL PRIMARY KEY,
    account_address VARCHAR(64) NOT NULL,
    owner VARCHAR(64) NOT NULL,
    mint VARCHAR(64) NOT NULL REFERENCES token_mints(mint_address) ON DELETE CASCADE,
    balance BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(owner, mint)
);

CREATE INDEX idx_token_accounts_owner ON token_accounts(owner);
CREATE INDEX idx_token_accounts_mint ON token_accounts(mint);
CREATE INDEX idx_token_accounts_balance ON token_accounts(mint, balance DESC);

-- ============================================================================
-- PRIVACY TRACKING
-- ============================================================================

-- Nullifiers (spent note markers)
CREATE TABLE nullifiers (
    nullifier VARCHAR(64) PRIMARY KEY,
    tx_id VARCHAR(64) NOT NULL REFERENCES transactions(tx_id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_nullifiers_tx ON nullifiers(tx_id);

-- Commitments (note commitments)
CREATE TABLE commitments (
    commitment VARCHAR(64) PRIMARY KEY,
    tx_id VARCHAR(64) NOT NULL REFERENCES transactions(tx_id) ON DELETE CASCADE,
    spent BOOLEAN NOT NULL DEFAULT FALSE,
    spent_tx_id VARCHAR(64) REFERENCES transactions(tx_id) ON DELETE SET NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_commitments_tx ON commitments(tx_id);
CREATE INDEX idx_commitments_unspent ON commitments(spent) WHERE NOT spent;

-- ============================================================================
-- PROGRAMS (Deployed smart contracts)
-- ============================================================================

CREATE TABLE programs (
    program_id VARCHAR(64) PRIMARY KEY,
    deployer VARCHAR(64) NOT NULL,
    deploy_tx_id VARCHAR(64) REFERENCES transactions(tx_id) ON DELETE SET NULL,
    code_size INTEGER NOT NULL,
    invoke_count BIGINT NOT NULL DEFAULT 0,
    last_invoked BIGINT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_programs_deployer ON programs(deployer);
CREATE INDEX idx_programs_invoke_count ON programs(invoke_count DESC);

-- ============================================================================
-- NETWORK STATISTICS
-- ============================================================================

-- Current network stats (single row)
CREATE TABLE network_stats (
    id INTEGER PRIMARY KEY DEFAULT 1 CHECK (id = 1),
    block_height BIGINT NOT NULL DEFAULT 0,
    total_transactions BIGINT NOT NULL DEFAULT 0,
    total_accounts BIGINT NOT NULL DEFAULT 0,
    total_validators BIGINT NOT NULL DEFAULT 0,
    active_validators BIGINT NOT NULL DEFAULT 0,
    atlas_total_supply BIGINT NOT NULL DEFAULT 0,
    atlas_staked BIGINT NOT NULL DEFAULT 0,
    shrug_total_supply BIGINT NOT NULL DEFAULT 0,
    shrug_burned BIGINT NOT NULL DEFAULT 0,
    avg_block_time DOUBLE PRECISION NOT NULL DEFAULT 0,
    tps_current DOUBLE PRECISION NOT NULL DEFAULT 0,
    tps_peak DOUBLE PRECISION NOT NULL DEFAULT 0,
    current_epoch BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Insert initial stats row
INSERT INTO network_stats (id) VALUES (1);

-- Hourly statistics
CREATE TABLE stats_hourly (
    hour BIGINT PRIMARY KEY, -- Unix timestamp of hour start
    block_count INTEGER NOT NULL DEFAULT 0,
    tx_count INTEGER NOT NULL DEFAULT 0,
    unique_senders INTEGER NOT NULL DEFAULT 0,
    total_fees BIGINT NOT NULL DEFAULT 0,
    avg_block_time DOUBLE PRECISION NOT NULL DEFAULT 0,
    tps_avg DOUBLE PRECISION NOT NULL DEFAULT 0,
    tps_peak DOUBLE PRECISION NOT NULL DEFAULT 0,
    -- Transaction type breakdown
    tx_public INTEGER NOT NULL DEFAULT 0,
    tx_private INTEGER NOT NULL DEFAULT 0,
    tx_stealth INTEGER NOT NULL DEFAULT 0,
    tx_stake INTEGER NOT NULL DEFAULT 0,
    tx_unstake INTEGER NOT NULL DEFAULT 0,
    tx_transfer INTEGER NOT NULL DEFAULT 0,
    tx_deploy INTEGER NOT NULL DEFAULT 0,
    tx_invoke INTEGER NOT NULL DEFAULT 0,
    tx_private_transfer INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_stats_hourly ON stats_hourly(hour DESC);

-- ============================================================================
-- INDEXER STATE
-- ============================================================================

-- Indexer checkpoint for crash recovery
CREATE TABLE indexer_state (
    id INTEGER PRIMARY KEY DEFAULT 1 CHECK (id = 1),
    last_indexed_height BIGINT NOT NULL DEFAULT 0,
    last_indexed_block_id VARCHAR(64),
    last_finalized_height BIGINT NOT NULL DEFAULT 0,
    is_syncing BOOLEAN NOT NULL DEFAULT FALSE,
    sync_started_at TIMESTAMP WITH TIME ZONE,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Insert initial indexer state
INSERT INTO indexer_state (id) VALUES (1);

-- ============================================================================
-- FUNCTIONS AND TRIGGERS
-- ============================================================================

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply updated_at trigger to relevant tables
CREATE TRIGGER update_accounts_updated_at BEFORE UPDATE ON accounts
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER update_validators_updated_at BEFORE UPDATE ON validators
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER update_epochs_updated_at BEFORE UPDATE ON epochs
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER update_token_mints_updated_at BEFORE UPDATE ON token_mints
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER update_token_accounts_updated_at BEFORE UPDATE ON token_accounts
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER update_programs_updated_at BEFORE UPDATE ON programs
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER update_network_stats_updated_at BEFORE UPDATE ON network_stats
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER update_indexer_state_updated_at BEFORE UPDATE ON indexer_state
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();
