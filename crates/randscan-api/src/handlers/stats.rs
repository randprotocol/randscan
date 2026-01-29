//! Statistics handlers

use crate::{error::AppError, state::AppState, ApiResult};
use axum::{
    extract::{Query, State},
    Json,
};
use randscan_core::{NetworkStats, StatsHourly, StatsQuery};
use randscan_db as db;

/// GET /api/v1/stats
pub async fn get_stats(State(state): State<AppState>) -> ApiResult<Json<NetworkStats>> {
    let stats = db::get_network_stats(state.db.inner()).await?;

    Ok(Json(NetworkStats {
        block_height: stats.block_height,
        total_transactions: stats.total_transactions,
        total_accounts: stats.total_accounts,
        total_validators: stats.total_validators,
        active_validators: stats.active_validators,
        atlas_total_supply: stats.atlas_total_supply,
        atlas_staked: stats.atlas_staked,
        shrug_total_supply: stats.shrug_total_supply,
        shrug_burned: stats.shrug_burned,
        avg_block_time: stats.avg_block_time,
        tps_current: stats.tps_current,
        tps_peak: stats.tps_peak,
        current_epoch: stats.current_epoch,
        updated_at: stats.updated_at.timestamp_millis(),
    }))
}

/// GET /api/v1/stats/hourly
pub async fn get_hourly_stats(
    State(state): State<AppState>,
    Query(query): Query<StatsQuery>,
) -> ApiResult<Json<Vec<StatsHourly>>> {
    let now = chrono::Utc::now().timestamp();
    let hour_ms = 3600 * 1000;

    let from = query.from_timestamp.unwrap_or(now - 24 * hour_ms); // Default last 24 hours
    let to = query.to_timestamp.unwrap_or(now);

    // Round to hour boundaries
    let from_hour = (from / hour_ms) * hour_ms;
    let to_hour = (to / hour_ms) * hour_ms;

    let stats = db::get_stats_hourly(state.db.inner(), from_hour, to_hour).await?;

    let results: Vec<StatsHourly> = stats
        .into_iter()
        .map(|s| StatsHourly {
            hour: s.hour,
            block_count: s.block_count,
            tx_count: s.tx_count,
            unique_senders: s.unique_senders,
            total_fees: s.total_fees,
            avg_block_time: s.avg_block_time,
            tps_avg: s.tps_avg,
            tps_peak: s.tps_peak,
            tx_public: s.tx_public,
            tx_private: s.tx_private,
            tx_stealth: s.tx_stealth,
            tx_stake: s.tx_stake,
            tx_unstake: s.tx_unstake,
            tx_transfer: s.tx_transfer,
            tx_deploy: s.tx_deploy,
            tx_invoke: s.tx_invoke,
            tx_private_transfer: s.tx_private_transfer,
        })
        .collect();

    Ok(Json(results))
}
