//! Block handlers

use crate::{error::AppError, state::AppState, ApiResult};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use randscan_core::{BlockDetail, BlockQuery, BlockSummary, PaginatedResponse, PaginationInfo};
use randscan_db as db;

/// GET /api/v1/blocks
pub async fn get_blocks(
    State(state): State<AppState>,
    Query(query): Query<BlockQuery>,
) -> ApiResult<Json<PaginatedResponse<BlockSummary>>> {
    let offset = query.pagination.offset();
    let limit = query.pagination.limit();

    let blocks = db::get_blocks(
        state.db.inner(),
        offset,
        limit,
        query.proposer.as_deref(),
        query.epoch,
        query.finalized,
    )
    .await?;

    let total = db::count_blocks(
        state.db.inner(),
        query.proposer.as_deref(),
        query.epoch,
        query.finalized,
    )
    .await?;

    let summaries: Vec<BlockSummary> = blocks
        .into_iter()
        .map(|b| BlockSummary {
            block_id: b.block_id,
            height: b.height as u64,
            view: b.view_number as u64,
            epoch: b.epoch as u64,
            parent_id: b.parent_id,
            proposer: b.proposer_id,
            timestamp: b.timestamp as u64,
            transaction_count: b.transaction_count,
            finalized: b.finalized,
        })
        .collect();

    Ok(Json(PaginatedResponse {
        data: summaries,
        pagination: PaginationInfo::new(query.pagination.page, query.pagination.limit, total),
    }))
}

/// GET /api/v1/blocks/latest
pub async fn get_latest_block(State(state): State<AppState>) -> ApiResult<Json<BlockSummary>> {
    let block = db::get_latest_block(state.db.inner()).await?;

    Ok(Json(BlockSummary {
        block_id: block.block_id,
        height: block.height as u64,
        view: block.view_number as u64,
        epoch: block.epoch as u64,
        parent_id: block.parent_id,
        proposer: block.proposer_id,
        timestamp: block.timestamp as u64,
        transaction_count: block.transaction_count,
        finalized: block.finalized,
    }))
}

/// GET /api/v1/blocks/:id
pub async fn get_block(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<BlockDetail>> {
    // Try to parse as height first
    let block = if let Ok(height) = id.parse::<i64>() {
        db::get_block_by_height(state.db.inner(), height).await?
    } else {
        db::get_block_by_id(state.db.inner(), &id).await?
    };

    // Get QC info
    let qc = db::get_qc_for_block(state.db.inner(), &block.block_id).await.ok();

    // Get transactions for this block
    let txs = db::get_transactions_for_block(state.db.inner(), &block.block_id).await?;
    let tx_ids: Vec<String> = txs.into_iter().map(|t| t.tx_id).collect();

    // Get QC signers
    let signers = if let Some(_) = &qc {
        db::get_qc_signers(state.db.inner(), &block.block_id)
            .await
            .map(|s| s.into_iter().map(|sig| sig.validator_id).collect())
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    Ok(Json(BlockDetail {
        block_id: block.block_id,
        height: block.height as u64,
        view: block.view_number as u64,
        epoch: block.epoch as u64,
        parent_id: block.parent_id,
        proposer: block.proposer_id,
        timestamp: block.timestamp as u64,
        transactions_root: block.transactions_root,
        state_root: block.state_root,
        supply_commitment: block.supply_commitment,
        transaction_count: block.transaction_count,
        transactions: tx_ids,
        finalized: block.finalized,
        qc_vote_type: qc.as_ref().map(|q| q.vote_type.clone()).unwrap_or_default(),
        qc_view: qc.as_ref().map(|q| q.view_number as u64).unwrap_or(0),
        qc_signers: signers,
    }))
}
