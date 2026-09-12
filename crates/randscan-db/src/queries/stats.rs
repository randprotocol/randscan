use crate::{NetworkStatsRow, Result};
use sqlx::PgPool;

pub struct StatsUpdate<'a> {
    pub chain_id: i64,
    pub symbol: &'a str,
    pub decimals: i16,
    pub height: i64,
    pub view: i64,
    pub total_transactions: i64,
    pub notes: i64,
    pub nullifiers: i64,
    pub validator_count: i64,
    pub active_validator_count: i64,
    pub total_stake: &'a str,
    pub total_supply: &'a str,
    pub pool_value: Option<&'a str>,
    pub program_count: i64,
    pub avg_block_time_ms: f64,
    pub peer_count: i32,
    pub mempool_size: i32,
    pub node_syncing: bool,
    pub faucet: bool,
    pub confidential: bool,
    pub current_leader: Option<&'a str>,
    pub tree_root: Option<&'a str>,
    pub hc_bundle: Option<&'a str>,
    pub epoch: Option<i64>,
    pub epoch_blocks: Option<i64>,
}

pub async fn update_network_stats(pool: &PgPool, s: &StatsUpdate<'_>) -> Result<()> {
    sqlx::query(
        "UPDATE network_stats SET chain_id = $1, symbol = $2, decimals = $3, height = $4, view = $5,
            total_transactions = $6, notes = $7, nullifiers = $8, validator_count = $9,
            active_validator_count = $10, total_stake = $11::numeric, total_supply = $12::numeric,
            pool_value = $13::numeric, program_count = $14, avg_block_time_ms = $15, peer_count = $16,
            mempool_size = $17, node_syncing = $18, faucet = $19, confidential = $20, current_leader = $21,
            tree_root = $22, hc_bundle = $23, epoch = $24, epoch_blocks = $25, updated_at = NOW()
         WHERE id = 1",
    )
    .bind(s.chain_id)
    .bind(s.symbol)
    .bind(s.decimals)
    .bind(s.height)
    .bind(s.view)
    .bind(s.total_transactions)
    .bind(s.notes)
    .bind(s.nullifiers)
    .bind(s.validator_count)
    .bind(s.active_validator_count)
    .bind(s.total_stake)
    .bind(s.total_supply)
    .bind(s.pool_value)
    .bind(s.program_count)
    .bind(s.avg_block_time_ms)
    .bind(s.peer_count)
    .bind(s.mempool_size)
    .bind(s.node_syncing)
    .bind(s.faucet)
    .bind(s.confidential)
    .bind(s.current_leader)
    .bind(s.tree_root)
    .bind(s.hc_bundle)
    .bind(s.epoch)
    .bind(s.epoch_blocks)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_network_stats(pool: &PgPool) -> Result<NetworkStatsRow> {
    Ok(sqlx::query_as::<_, NetworkStatsRow>(
        "SELECT chain_id, symbol, decimals, height, view, total_transactions, notes, nullifiers,
                validator_count, active_validator_count, total_stake::text AS total_stake,
                total_supply::text AS total_supply, pool_value::text AS pool_value, program_count,
                avg_block_time_ms, peer_count, mempool_size, node_syncing, faucet, confidential,
                current_leader, tree_root, hc_bundle, epoch, epoch_blocks, updated_at
         FROM network_stats WHERE id = 1",
    )
    .fetch_one(pool)
    .await?)
}
