use crate::{state::AppState, ApiResult};
use axum::{extract::State, Json};
use randscan_core::NetworkStats;
use randscan_db as db;

/// GET /api/v1/stats
pub async fn get_stats(State(state): State<AppState>) -> ApiResult<Json<NetworkStats>> {
    Ok(Json(db::get_network_stats(state.db.inner()).await?.into()))
}
