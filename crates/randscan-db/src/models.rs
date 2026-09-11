//! Row types. NUMERIC columns are selected as `::text` and carried as `String`.

use chrono::{DateTime, Utc};
use randscan_core::{
    BlockSummary, GeoInfo, NetworkStats, ProgramSummary, Receipt, ReceiptEffect,
    TransactionSummary, TxKind, Validator,
};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct BlockRow {
    pub hash: String,
    pub height: i64,
    pub view: i64,
    pub parent: String,
    pub proposer: String,
    pub timestamp_ms: i64,
    pub tx_root: String,
    pub state_root: String,
    pub justify_view: i64,
    pub tx_count: i32,
}

impl From<BlockRow> for BlockSummary {
    fn from(b: BlockRow) -> Self {
        BlockSummary {
            hash: b.hash,
            height: b.height,
            view: b.view,
            parent: b.parent,
            proposer: b.proposer,
            timestamp_ms: b.timestamp_ms,
            tx_count: b.tx_count,
            justify_view: b.justify_view,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct TxRow {
    pub hash: String,
    pub block_hash: String,
    pub height: i64,
    pub tx_index: i32,
    pub sender: String,
    pub nonce: i64,
    pub fee: String,
    pub kind: String,
    pub chain_id: i64,
    pub timestamp_ms: i64,
    pub to_address: Option<String>,
    pub amount: Option<String>,
    pub program_id: Option<String>,
    pub base_pc: Option<i64>,
    pub words_len: Option<i64>,
    pub proof_len: Option<i64>,
    pub recipients: Vec<String>,
    pub asset: Option<String>,
    pub bridge_amount: Option<String>,
    pub to_chain: Option<i32>,
    pub bridge_to: Option<String>,
    pub bridge_fee: Option<String>,
}

impl From<TxRow> for TransactionSummary {
    fn from(t: TxRow) -> Self {
        TransactionSummary {
            hash: t.hash,
            height: t.height,
            block_hash: t.block_hash,
            tx_index: t.tx_index,
            sender: t.sender,
            nonce: t.nonce,
            fee: t.fee,
            kind: TxKind::parse_lossy(&t.kind),
            timestamp_ms: t.timestamp_ms,
            to: t.to_address,
            amount: t.amount,
            program: t.program_id,
        }
    }
}

/// Transaction row joined with the account role.
#[derive(Debug, Clone, FromRow)]
pub struct AccountTxRow {
    #[sqlx(flatten)]
    pub tx: TxRow,
    pub role: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct ReceiptRow {
    pub tx_hash: String,
    pub program: String,
    pub tier: i32,
    pub outputs: Vec<i64>,
    pub effect_to: Option<String>,
    pub effect_amount: Option<String>,
    pub height: i64,
    pub tx_index: i32,
}

impl From<ReceiptRow> for Receipt {
    fn from(r: ReceiptRow) -> Self {
        let effect = match (r.effect_to, r.effect_amount) {
            (Some(to), Some(amount)) => Some(ReceiptEffect { to, amount }),
            _ => None,
        };
        Receipt {
            tx: r.tx_hash,
            program: r.program,
            tier: r.tier,
            outputs: r.outputs,
            effect,
            height: r.height,
            index: r.tx_index,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct AccountRow {
    pub address: String,
    pub balance: String,
    pub nonce: i64,
    pub tx_count: i64,
    pub first_seen_height: i64,
    pub last_seen_height: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct ValidatorRow {
    pub address: String,
    pub stake: String,
    pub sort_index: i32,
    pub blocks_proposed: i64,
    pub last_proposed_height: Option<i64>,
    pub last_proposed_timestamp_ms: Option<i64>,
}

impl ValidatorRow {
    pub fn into_validator(self, total_stake: u128) -> Validator {
        let stake: u128 = self.stake.parse().unwrap_or(0);
        let share_percent = if total_stake == 0 {
            0.0
        } else {
            (stake as f64 / total_stake as f64) * 100.0
        };
        Validator {
            address: self.address,
            stake: self.stake,
            share_percent,
            blocks_proposed: self.blocks_proposed,
            last_proposed_height: self.last_proposed_height,
            last_proposed_timestamp_ms: self.last_proposed_timestamp_ms,
            sort_index: self.sort_index,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct ProgramRow {
    pub id: String,
    pub deployer: String,
    pub deploy_tx: String,
    pub deployed_at_height: i64,
    pub base_pc: i64,
    pub words_len: i64,
    pub code_hash: String,
    pub call_count: i64,
    pub last_called_height: Option<i64>,
}

impl From<ProgramRow> for ProgramSummary {
    fn from(p: ProgramRow) -> Self {
        ProgramSummary {
            id: p.id,
            deployer: p.deployer,
            deploy_tx: p.deploy_tx,
            deployed_at_height: p.deployed_at_height,
            base_pc: p.base_pc,
            words_len: p.words_len,
            code_hash: p.code_hash,
            call_count: p.call_count,
            last_called_height: p.last_called_height,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct NetworkStatsRow {
    pub chain_id: i64,
    pub symbol: String,
    pub decimals: i16,
    pub height: i64,
    pub view: i64,
    pub total_transactions: i64,
    pub total_accounts: i64,
    pub validator_count: i64,
    pub total_stake: String,
    pub total_supply: String,
    pub program_count: i64,
    pub avg_block_time_ms: f64,
    pub peer_count: i32,
    pub mempool_size: i32,
    pub node_syncing: bool,
    pub faucet: bool,
    pub confidential: bool,
    pub current_leader: Option<String>,
    pub updated_at: DateTime<Utc>,
}

impl From<NetworkStatsRow> for NetworkStats {
    fn from(s: NetworkStatsRow) -> Self {
        NetworkStats {
            chain_id: s.chain_id,
            symbol: s.symbol,
            decimals: s.decimals,
            height: s.height,
            view: s.view,
            total_transactions: s.total_transactions,
            total_accounts: s.total_accounts,
            validator_count: s.validator_count,
            total_stake: s.total_stake,
            total_supply: s.total_supply,
            program_count: s.program_count,
            avg_block_time_ms: s.avg_block_time_ms,
            peer_count: s.peer_count,
            mempool_size: s.mempool_size,
            node_syncing: s.node_syncing,
            faucet: s.faucet,
            confidential: s.confidential,
            current_leader: s.current_leader,
            updated_at: s.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct IndexerStateRow {
    pub next_height: i64,
    pub last_indexed_hash: Option<String>,
    pub is_syncing: bool,
    /// Chain id of the indexed data; `None` before the indexer first reached a node.
    pub chain_id: Option<i64>,
}

#[derive(Debug, Clone, FromRow)]
pub struct NodeGeoRow {
    pub ip: String,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub org: Option<String>,
    pub ok: bool,
    pub updated_at: DateTime<Utc>,
}

impl NodeGeoRow {
    pub fn geo(&self) -> Option<GeoInfo> {
        match (self.ok, self.lat, self.lon) {
            (true, Some(lat), Some(lon)) => Some(GeoInfo {
                lat,
                lon,
                city: self.city.clone(),
                region: self.region.clone(),
                country: self.country.clone(),
                country_code: self.country_code.clone(),
                org: self.org.clone(),
            }),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct UserRow {
    pub id: i64,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow)]
pub struct SessionRow {
    pub token_hash: String,
    pub user_id: i64,
    pub expires_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct ApiKeyRow {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub prefix: String,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub request_count: i64,
    pub revoked_at: Option<DateTime<Utc>>,
}
