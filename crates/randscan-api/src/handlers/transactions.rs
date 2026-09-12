use crate::{error::AppError, state::AppState, ApiResult};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use randscan_core::{
    classify_query, LimitQuery, PaginatedResponse, PaginationInfo, QueryKind, TransactionDetail,
    TransactionQuery, TransactionSummary, TxKind,
};
use randscan_db as db;

/// GET /api/v1/transactions
pub async fn list_transactions(
    State(state): State<AppState>,
    Query(q): Query<TransactionQuery>,
) -> ApiResult<Json<PaginatedResponse<TransactionSummary>>> {
    let kind = match q.kind.as_deref().filter(|k| !k.is_empty() && *k != "all") {
        Some(k) => Some(
            TxKind::parse(k).ok_or_else(|| AppError::BadRequest(format!("unknown kind {}", k)))?,
        ),
        None => None,
    };
    let program = match q.program.as_deref().filter(|p| !p.is_empty()) {
        Some(p) => match classify_query(p) {
            QueryKind::Hash(h) => Some(h),
            _ => {
                return Err(AppError::BadRequest(
                    "program must be 64 hex characters".into(),
                ))
            }
        },
        None => None,
    };
    let filter = db::TxFilter {
        kind: kind.map(|k| k.as_str()),
        height: q.height,
        validator: q.validator.as_deref().filter(|v| !v.is_empty()),
        program: program.as_deref(),
    };
    let pool = state.db.inner();
    let pg = q.pagination();
    let p = &pg;
    let rows = db::list_transactions(pool, p.offset(), p.limit(), &filter).await?;
    let total = db::count_transactions(pool, &filter).await?;
    Ok(Json(PaginatedResponse {
        data: rows.into_iter().map(Into::into).collect(),
        pagination: PaginationInfo::new(p, total),
    }))
}

/// GET /api/v1/transactions/latest
pub async fn latest_transactions(
    State(state): State<AppState>,
    Query(q): Query<LimitQuery>,
) -> ApiResult<Json<Vec<TransactionSummary>>> {
    let rows = db::get_latest_transactions(state.db.inner(), q.limit()).await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

/// GET /api/v1/transactions/:hash
pub async fn get_transaction(
    State(state): State<AppState>,
    Path(hash): Path<String>,
) -> ApiResult<Json<TransactionDetail>> {
    let hash = match classify_query(&hash) {
        QueryKind::Hash(h) => h,
        _ => {
            return Err(AppError::BadRequest(
                "transaction hash must be 64 hex characters".into(),
            ))
        }
    };
    let pool = state.db.inner();
    let row = db::get_transaction(pool, &hash)
        .await?
        .ok_or_else(|| AppError::NotFound("transaction".into()))?;
    let receipt = if row.tx.kind == "call" {
        db::get_receipt(pool, &hash).await?.map(Into::into)
    } else {
        None
    };
    Ok(Json(row.into_detail(receipt)))
}
