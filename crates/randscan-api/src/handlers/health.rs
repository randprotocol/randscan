use crate::{state::AppState, ApiResult};
use axum::{extract::State, Json};
use randscan_core::{HealthResponse, IndexerHealth};
use randscan_db as db;

/// GET /api/v1/health
pub async fn health(State(state): State<AppState>) -> ApiResult<Json<HealthResponse>> {
    let db_ok = db::get_network_stats(state.db.inner()).await.is_ok();
    let s = state.indexer.get_state().await;
    let lag = (s.node_height - s.current_height).max(0);
    let indexer = IndexerHealth {
        connected: s.is_connected,
        synced: s.is_connected && !s.is_syncing && lag <= 2,
        current_height: s.current_height,
        node_height: s.node_height,
        lag,
    };
    let status = if db_ok && indexer.connected && indexer.synced {
        "healthy"
    } else if db_ok {
        "degraded"
    } else {
        "unhealthy"
    };
    Ok(Json(HealthResponse {
        status: status.into(),
        version: env!("CARGO_PKG_VERSION").into(),
        database: db_ok,
        indexer,
    }))
}
