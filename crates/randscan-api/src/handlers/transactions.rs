//! Transaction handlers

use crate::{error::AppError, state::AppState, ApiResult};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use randscan_core::{
    PaginatedResponse, PaginationInfo, PayloadType, TransactionDetail, TransactionQuery,
    TransactionSummary,
};
use randscan_db as db;

/// GET /api/v1/transactions
pub async fn get_transactions(
    State(state): State<AppState>,
    Query(query): Query<TransactionQuery>,
) -> ApiResult<Json<PaginatedResponse<TransactionSummary>>> {
    let offset = query.pagination.offset();
    let limit = query.pagination.limit();

    let transactions = db::get_transactions(
        state.db.inner(),
        offset,
        limit,
        query.sender.as_deref(),
        query.block_id.as_deref(),
        query.payload_type.as_deref(),
        query.status.as_deref(),
        query.from_timestamp,
        query.to_timestamp,
    )
    .await?;

    let total = db::count_transactions(
        state.db.inner(),
        query.sender.as_deref(),
        query.block_id.as_deref(),
        query.payload_type.as_deref(),
        query.status.as_deref(),
    )
    .await?;

    let summaries: Vec<TransactionSummary> = transactions
        .into_iter()
        .map(|tx| {
            let payload_type = PayloadType::from_str(&tx.payload_type).unwrap_or(PayloadType::Public);
            TransactionSummary {
                tx_id: tx.tx_id,
                block_id: tx.block_id,
                block_height: tx.block_height,
                sender: tx.sender,
                nonce: tx.nonce,
                fee: tx.fee,
                payload_type: tx.payload_type.clone(),
                status: tx.status,
                timestamp: tx.timestamp,
                privacy_level: payload_type.privacy_level().as_str().to_string(),
            }
        })
        .collect();

    Ok(Json(PaginatedResponse {
        data: summaries,
        pagination: PaginationInfo::new(query.pagination.page, query.pagination.limit, total),
    }))
}

/// GET /api/v1/transactions/:id
pub async fn get_transaction(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<TransactionDetail>> {
    let tx = db::get_transaction_by_id(state.db.inner(), &id).await?;

    let payload_type = PayloadType::from_str(&tx.payload_type).unwrap_or(PayloadType::Public);

    // Get nullifier/commitment counts for privacy transactions
    let (nullifier_count, commitment_count) = if payload_type.is_private() {
        let nullifiers = db::get_nullifiers_for_tx(state.db.inner(), &id)
            .await
            .map(|n| n.len())
            .unwrap_or(0);
        let commitments = db::get_commitments_for_tx(state.db.inner(), &id)
            .await
            .map(|c| c.len())
            .unwrap_or(0);
        (nullifiers as i32, commitments as i32)
    } else {
        (0, 0)
    };

    Ok(Json(TransactionDetail {
        tx_id: tx.tx_id,
        block_id: tx.block_id,
        block_height: tx.block_height,
        sender: tx.sender,
        nonce: tx.nonce,
        compute_budget: tx.compute_budget,
        fee: tx.fee,
        payload_type: tx.payload_type.clone(),
        status: tx.status,
        timestamp: tx.timestamp,
        signature: tx.signature,
        privacy_level: payload_type.privacy_level().as_str().to_string(),
        payload_data: None, // Would need to fetch from type-specific tables
        nullifier_count,
        commitment_count,
    }))
}
