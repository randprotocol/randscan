//! Health check handler

use crate::{state::AppState, ApiResult};
use axum::{extract::State, Json};
use randscan_core::{HealthResponse, IndexerHealth};
use randscan_db as db;

/// GET /api/v1/health
pub async fn health(State(state): State<AppState>) -> ApiResult<Json<HealthResponse>> {
    let db_ok = db::get_network_stats(state.db.inner()).await.is_ok();

    let indexer_health = if let Some(ref indexer) = state.indexer {
        let indexer_state = indexer.get_state().await;
        IndexerHealth {
            connected: indexer_state.is_connected,
            synced: !indexer_state.is_syncing,
            current_height: indexer_state.current_height as i64,
            node_height: indexer_state.node_height as i64,
            lag: (indexer_state.node_height - indexer_state.current_height) as i64,
        }
    } else {
        IndexerHealth {
            connected: false,
            synced: false,
            current_height: 0,
            node_height: 0,
            lag: 0,
        }
    };

    let status = if db_ok && indexer_health.connected && indexer_health.synced {
        "healthy"
    } else if db_ok {
        "degraded"
    } else {
        "unhealthy"
    };

    Ok(Json(HealthResponse {
        status: status.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        database: db_ok,
        indexer: indexer_health,
    }))
}
