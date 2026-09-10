use crate::{error::AppError, state::AppState, ApiResult};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use randscan_core::{
    classify_query, BlockDetail, BlockQuery, BlockSummary, LimitQuery, PaginatedResponse,
    PaginationInfo, QueryKind,
};
use randscan_db as db;

/// GET /api/v1/blocks
pub async fn list_blocks(
    State(state): State<AppState>,
    Query(q): Query<BlockQuery>,
) -> ApiResult<Json<PaginatedResponse<BlockSummary>>> {
    let pool = state.db.inner();
    let pg = q.pagination();
    let p = &pg;
    let rows = db::list_blocks(pool, p.offset(), p.limit(), q.proposer.as_deref()).await?;
    let total = db::count_blocks(pool, q.proposer.as_deref()).await?;
    Ok(Json(PaginatedResponse {
        data: rows.into_iter().map(Into::into).collect(),
        pagination: PaginationInfo::new(p, total),
    }))
}

/// GET /api/v1/blocks/latest
pub async fn latest_blocks(
    State(state): State<AppState>,
    Query(q): Query<LimitQuery>,
) -> ApiResult<Json<Vec<BlockSummary>>> {
    let rows = db::get_latest_blocks(state.db.inner(), q.limit()).await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

/// GET /api/v1/blocks/:id (height or hash)
pub async fn get_block(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<BlockDetail>> {
    let pool = state.db.inner();
    let row = match classify_query(&id) {
        QueryKind::Height(h) => db::get_block_by_height(pool, h).await?,
        QueryKind::Hash(h) => db::get_block_by_hash(pool, &h).await?,
        _ => {
            return Err(AppError::BadRequest(
                "block id must be a height or a 64-hex hash".into(),
            ))
        }
    };
    let row = row.ok_or_else(|| AppError::NotFound("block".into()))?;
    let txs = db::get_transactions_for_block(pool, &row.hash).await?;
    let tx_root = row.tx_root.clone();
    let state_root = row.state_root.clone();
    Ok(Json(BlockDetail {
        summary: row.into(),
        tx_root,
        state_root,
        transactions: txs.into_iter().map(Into::into).collect(),
    }))
}
