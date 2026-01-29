//! Validator handlers

use crate::{error::AppError, state::AppState, ApiResult};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use randscan_core::{
    PaginatedResponse, PaginationInfo, StakeHistoryEntry, Validator, ValidatorBlock,
    ValidatorDetail, ValidatorQuery,
};
use randscan_db as db;

/// GET /api/v1/validators
pub async fn get_validators(
    State(state): State<AppState>,
    Query(query): Query<ValidatorQuery>,
) -> ApiResult<Json<PaginatedResponse<Validator>>> {
    let offset = query.pagination.offset();
    let limit = query.pagination.limit();

    let validators = db::get_validators(
        state.db.inner(),
        offset,
        limit,
        query.active,
        query.sort_by.as_deref(),
    )
    .await?;

    let total = db::count_validators(state.db.inner(), query.active).await?;

    let results: Vec<Validator> = validators
        .into_iter()
        .map(|v| Validator {
            validator_id: v.validator_id,
            pubkey: v.pubkey,
            stake: v.stake,
            commission_rate: v.commission_rate,
            is_active: v.is_active,
            blocks_produced: v.blocks_produced,
            blocks_skipped: v.blocks_skipped,
            last_vote_height: v.last_vote_height,
            uptime_percentage: v.uptime_percentage,
            first_seen: v.first_seen,
            last_seen: v.last_seen,
        })
        .collect();

    Ok(Json(PaginatedResponse {
        data: results,
        pagination: PaginationInfo::new(query.pagination.page, query.pagination.limit, total),
    }))
}

/// GET /api/v1/validators/:id
pub async fn get_validator(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<ValidatorDetail>> {
    let validator = db::get_validator_by_id(state.db.inner(), &id).await?;

    // Get recent blocks by this validator
    let recent_blocks = db::get_blocks_by_proposer(state.db.inner(), &id, 10)
        .await
        .unwrap_or_default();

    let blocks: Vec<ValidatorBlock> = recent_blocks
        .into_iter()
        .map(|b| ValidatorBlock {
            block_id: b.block_id,
            height: b.height,
            timestamp: b.timestamp,
            transaction_count: b.transaction_count,
        })
        .collect();

    // Get stake history
    let history = db::get_stake_history(state.db.inner(), &id, 30)
        .await
        .unwrap_or_default();

    let stake_history: Vec<StakeHistoryEntry> = history
        .into_iter()
        .map(|h| StakeHistoryEntry {
            validator_id: h.validator_id,
            epoch: h.epoch,
            stake: h.stake,
            stake_display: h.stake as f64 / 1_000_000_000.0,
            delegators: h.delegators,
            rewards: h.rewards,
            timestamp: h.timestamp,
        })
        .collect();

    // Calculate skip rate
    let total_slots = validator.blocks_produced + validator.blocks_skipped;
    let skip_rate = if total_slots > 0 {
        validator.blocks_skipped as f64 / total_slots as f64 * 100.0
    } else {
        0.0
    };

    Ok(Json(ValidatorDetail {
        validator_id: validator.validator_id,
        pubkey: validator.pubkey,
        stake: validator.stake,
        stake_display: validator.stake as f64 / 1_000_000_000.0,
        commission_rate: validator.commission_rate,
        is_active: validator.is_active,
        blocks_produced: validator.blocks_produced,
        blocks_skipped: validator.blocks_skipped,
        skip_rate,
        last_vote_height: validator.last_vote_height,
        uptime_percentage: validator.uptime_percentage,
        first_seen: validator.first_seen,
        last_seen: validator.last_seen,
        recent_blocks: blocks,
        stake_history,
        avg_block_time: None,
        total_rewards: 0, // Would calculate from history
    }))
}
