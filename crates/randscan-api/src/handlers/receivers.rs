use crate::{error::AppError, state::AppState, ApiResult};
use axum::{
    extract::{Path, State},
    Json,
};
use randprotocol_core::ReceiverId;
use randscan_core::ReceiverRecordView;
use randscan_db as db;

/// GET /api/v1/receivers/:address — the current record (highest version) for a short shielded
/// address. 400 when `address` does not parse as a `rand1…` id, 404 when it parses but nothing
/// has ever registered it.
pub async fn get_receiver(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> ApiResult<Json<ReceiverRecordView>> {
    let id = ReceiverId::parse(&address)
        .map_err(|e| AppError::BadRequest(format!("not a shielded address: {e}")))?;
    let pool = state.db.inner();
    let row = db::get_receiver(pool, &id.to_string())
        .await?
        .ok_or_else(|| AppError::NotFound("receiver".into()))?;
    Ok(Json(row.into()))
}

/// GET /api/v1/receivers/:address/history — every version ever published for this address,
/// newest first. An address that parses but was never registered serves an empty list (not
/// 404: the list, unlike the current record, has a well-defined empty answer).
pub async fn get_receiver_history(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> ApiResult<Json<Vec<ReceiverRecordView>>> {
    let id = ReceiverId::parse(&address)
        .map_err(|e| AppError::BadRequest(format!("not a shielded address: {e}")))?;
    let pool = state.db.inner();
    let rows = db::receiver_history(pool, &id.to_string()).await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}
