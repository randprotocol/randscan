//! Statistics queries

use crate::{EpochRow, NetworkStatsRow, Result, StatsHourlyRow};
use sqlx::PgPool;

/// Get network stats
pub async fn get_network_stats(pool: &PgPool) -> Result<NetworkStatsRow> {
    sqlx::query_as::<_, NetworkStatsRow>("SELECT * FROM network_stats WHERE id = 1")
        .fetch_one(pool)
        .await
        .map_err(Into::into)
}

/// Update network stats
pub async fn update_network_stats(
    pool: &PgPool,
    block_height: i64,
    total_transactions: i64,
    total_accounts: i64,
    total_validators: i64,
    active_validators: i64,
    atlas_total_supply: i64,
    atlas_staked: i64,
    shrug_total_supply: i64,
    shrug_burned: i64,
    avg_block_time: f64,
    tps_current: f64,
    tps_peak: f64,
    current_epoch: i64,
) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE network_stats SET
            block_height = $1,
            total_transactions = $2,
            total_accounts = $3,
            total_validators = $4,
            active_validators = $5,
            atlas_total_supply = $6,
            atlas_staked = $7,
            shrug_total_supply = $8,
            shrug_burned = $9,
            avg_block_time = $10,
            tps_current = $11,
            tps_peak = GREATEST(tps_peak, $12),
            current_epoch = $13
        WHERE id = 1
        "#,
    )
    .bind(block_height)
    .bind(total_transactions)
    .bind(total_accounts)
    .bind(total_validators)
    .bind(active_validators)
    .bind(atlas_total_supply)
    .bind(atlas_staked)
    .bind(shrug_total_supply)
    .bind(shrug_burned)
    .bind(avg_block_time)
    .bind(tps_current)
    .bind(tps_peak)
    .bind(current_epoch)
    .execute(pool)
    .await?;

    Ok(())
}

/// Increment network stats counters
pub async fn increment_network_stats(
    pool: &PgPool,
    blocks: i64,
    transactions: i64,
    accounts: i64,
) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE network_stats SET
            block_height = block_height + $1,
            total_transactions = total_transactions + $2,
            total_accounts = total_accounts + $3
        WHERE id = 1
        "#,
    )
    .bind(blocks)
    .bind(transactions)
    .bind(accounts)
    .execute(pool)
    .await?;

    Ok(())
}

/// Insert or update hourly stats
pub async fn upsert_stats_hourly(
    pool: &PgPool,
    hour: i64,
    block_count: i32,
    tx_count: i32,
    unique_senders: i32,
    total_fees: i64,
    avg_block_time: f64,
    tps_avg: f64,
    tps_peak: f64,
    tx_public: i32,
    tx_private: i32,
    tx_stealth: i32,
    tx_stake: i32,
    tx_unstake: i32,
    tx_transfer: i32,
    tx_deploy: i32,
    tx_invoke: i32,
    tx_private_transfer: i32,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO stats_hourly (
            hour, block_count, tx_count, unique_senders, total_fees,
            avg_block_time, tps_avg, tps_peak,
            tx_public, tx_private, tx_stealth, tx_stake, tx_unstake,
            tx_transfer, tx_deploy, tx_invoke, tx_private_transfer
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
        ON CONFLICT (hour) DO UPDATE SET
            block_count = stats_hourly.block_count + EXCLUDED.block_count,
            tx_count = stats_hourly.tx_count + EXCLUDED.tx_count,
            unique_senders = GREATEST(stats_hourly.unique_senders, EXCLUDED.unique_senders),
            total_fees = stats_hourly.total_fees + EXCLUDED.total_fees,
            avg_block_time = EXCLUDED.avg_block_time,
            tps_avg = EXCLUDED.tps_avg,
            tps_peak = GREATEST(stats_hourly.tps_peak, EXCLUDED.tps_peak),
            tx_public = stats_hourly.tx_public + EXCLUDED.tx_public,
            tx_private = stats_hourly.tx_private + EXCLUDED.tx_private,
            tx_stealth = stats_hourly.tx_stealth + EXCLUDED.tx_stealth,
            tx_stake = stats_hourly.tx_stake + EXCLUDED.tx_stake,
            tx_unstake = stats_hourly.tx_unstake + EXCLUDED.tx_unstake,
            tx_transfer = stats_hourly.tx_transfer + EXCLUDED.tx_transfer,
            tx_deploy = stats_hourly.tx_deploy + EXCLUDED.tx_deploy,
            tx_invoke = stats_hourly.tx_invoke + EXCLUDED.tx_invoke,
            tx_private_transfer = stats_hourly.tx_private_transfer + EXCLUDED.tx_private_transfer
        "#,
    )
    .bind(hour)
    .bind(block_count)
    .bind(tx_count)
    .bind(unique_senders)
    .bind(total_fees)
    .bind(avg_block_time)
    .bind(tps_avg)
    .bind(tps_peak)
    .bind(tx_public)
    .bind(tx_private)
    .bind(tx_stealth)
    .bind(tx_stake)
    .bind(tx_unstake)
    .bind(tx_transfer)
    .bind(tx_deploy)
    .bind(tx_invoke)
    .bind(tx_private_transfer)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get hourly stats in range
pub async fn get_stats_hourly(
    pool: &PgPool,
    from_hour: i64,
    to_hour: i64,
) -> Result<Vec<StatsHourlyRow>> {
    sqlx::query_as::<_, StatsHourlyRow>(
        "SELECT * FROM stats_hourly WHERE hour >= $1 AND hour <= $2 ORDER BY hour",
    )
    .bind(from_hour)
    .bind(to_hour)
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

/// Get recent hourly stats
pub async fn get_recent_stats_hourly(pool: &PgPool, hours: i32) -> Result<Vec<StatsHourlyRow>> {
    sqlx::query_as::<_, StatsHourlyRow>(
        "SELECT * FROM stats_hourly ORDER BY hour DESC LIMIT $1",
    )
    .bind(hours)
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

/// Insert or update epoch
pub async fn upsert_epoch(
    pool: &PgPool,
    epoch: i64,
    start_height: i64,
    end_height: Option<i64>,
    start_timestamp: i64,
    end_timestamp: Option<i64>,
    block_count: i32,
    tx_count: i32,
    validator_count: i32,
    total_stake: i64,
    total_rewards: i64,
    is_current: bool,
) -> Result<()> {
    // If this is current epoch, mark others as not current
    if is_current {
        sqlx::query("UPDATE epochs SET is_current = FALSE WHERE is_current = TRUE")
            .execute(pool)
            .await?;
    }

    sqlx::query(
        r#"
        INSERT INTO epochs (
            epoch, start_height, end_height, start_timestamp, end_timestamp,
            block_count, tx_count, validator_count, total_stake, total_rewards, is_current
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        ON CONFLICT (epoch) DO UPDATE SET
            end_height = COALESCE(EXCLUDED.end_height, epochs.end_height),
            end_timestamp = COALESCE(EXCLUDED.end_timestamp, epochs.end_timestamp),
            block_count = EXCLUDED.block_count,
            tx_count = EXCLUDED.tx_count,
            validator_count = EXCLUDED.validator_count,
            total_stake = EXCLUDED.total_stake,
            total_rewards = EXCLUDED.total_rewards,
            is_current = EXCLUDED.is_current
        "#,
    )
    .bind(epoch)
    .bind(start_height)
    .bind(end_height)
    .bind(start_timestamp)
    .bind(end_timestamp)
    .bind(block_count)
    .bind(tx_count)
    .bind(validator_count)
    .bind(total_stake)
    .bind(total_rewards)
    .bind(is_current)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get epoch by number
pub async fn get_epoch(pool: &PgPool, epoch: i64) -> Result<EpochRow> {
    sqlx::query_as::<_, EpochRow>("SELECT * FROM epochs WHERE epoch = $1")
        .bind(epoch)
        .fetch_one(pool)
        .await
        .map_err(Into::into)
}

/// Get current epoch
pub async fn get_current_epoch(pool: &PgPool) -> Result<EpochRow> {
    sqlx::query_as::<_, EpochRow>("SELECT * FROM epochs WHERE is_current = TRUE")
        .fetch_one(pool)
        .await
        .map_err(Into::into)
}

/// Get recent epochs
pub async fn get_recent_epochs(pool: &PgPool, limit: i32) -> Result<Vec<EpochRow>> {
    sqlx::query_as::<_, EpochRow>("SELECT * FROM epochs ORDER BY epoch DESC LIMIT $1")
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(Into::into)
}

/// Increment epoch counters
pub async fn increment_epoch_counters(
    pool: &PgPool,
    epoch: i64,
    blocks: i32,
    transactions: i32,
) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE epochs SET
            block_count = block_count + $2,
            tx_count = tx_count + $3
        WHERE epoch = $1
        "#,
    )
    .bind(epoch)
    .bind(blocks)
    .bind(transactions)
    .execute(pool)
    .await?;

    Ok(())
}
