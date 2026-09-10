use crate::{NetworkStatsRow, Result};
use sqlx::PgPool;

pub struct StatsUpdate<'a> {
    pub chain_id: i64,
    pub symbol: &'a str,
    pub decimals: i16,
    pub height: i64,
    pub view: i64,
    pub total_transactions: i64,
    pub total_accounts: i64,
    pub validator_count: i64,
    pub total_stake: &'a str,
    pub total_supply: &'a str,
    pub program_count: i64,
    pub avg_block_time_ms: f64,
    pub peer_count: i32,
    pub mempool_size: i32,
    pub node_syncing: bool,
    pub faucet: bool,
    pub confidential: bool,
    pub current_leader: Option<&'a str>,
}

pub async fn update_network_stats(pool: &PgPool, s: &StatsUpdate<'_>) -> Result<()> {
    sqlx::query(
        "UPDATE network_stats SET chain_id = $1, symbol = $2, decimals = $3, height = $4, view = $5,
            total_transactions = $6, total_accounts = $7, validator_count = $8, total_stake = $9::numeric,
            total_supply = $10::numeric, program_count = $11, avg_block_time_ms = $12, peer_count = $13,
            mempool_size = $14, node_syncing = $15, faucet = $16, confidential = $17, current_leader = $18,
            updated_at = NOW()
         WHERE id = 1",
    )
    .bind(s.chain_id)
    .bind(s.symbol)
    .bind(s.decimals)
    .bind(s.height)
    .bind(s.view)
    .bind(s.total_transactions)
    .bind(s.total_accounts)
    .bind(s.validator_count)
    .bind(s.total_stake)
    .bind(s.total_supply)
    .bind(s.program_count)
    .bind(s.avg_block_time_ms)
    .bind(s.peer_count)
    .bind(s.mempool_size)
    .bind(s.node_syncing)
    .bind(s.faucet)
    .bind(s.confidential)
    .bind(s.current_leader)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_network_stats(pool: &PgPool) -> Result<NetworkStatsRow> {
    Ok(sqlx::query_as::<_, NetworkStatsRow>(
        "SELECT chain_id, symbol, decimals, height, view, total_transactions, total_accounts, validator_count,
                total_stake::text AS total_stake, total_supply::text AS total_supply, program_count, avg_block_time_ms,
                peer_count, mempool_size, node_syncing, faucet, confidential, current_leader, updated_at
         FROM network_stats WHERE id = 1",
    )
    .fetch_one(pool)
    .await?)
}
