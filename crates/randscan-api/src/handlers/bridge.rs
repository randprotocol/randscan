use crate::{error::AppError, state::AppState, ApiResult};
use axum::{extract::State, Json};
use randscan_core::{BridgeState, Supply};

/// GET /api/v1/bridge — the bridge's public state as last read from the node.
pub async fn get_bridge(State(state): State<AppState>) -> ApiResult<Json<BridgeState>> {
    match state.indexer.bridge().await {
        Some(b) => Ok(Json(b)),
        // Not refreshed yet (the indexer has not reached the node): report a disabled bridge
        // rather than an error, which is also what a chain without one reports.
        None => Ok(Json(BridgeState {
            enabled: false,
            emitter: None,
            emitters: Default::default(),
            guardian_set_index: None,
            guardians: vec![],
            burn_sequence: None,
            next_index: None,
            assets: vec![],
        })),
    }
}

/// GET /api/v1/supply — the node's supply audit (phase S2); 404 on a node without it.
pub async fn get_supply(State(state): State<AppState>) -> ApiResult<Json<Supply>> {
    state
        .indexer
        .supply()
        .await
        .map(Json)
        .ok_or_else(|| AppError::NotFound("supply audit".into()))
}
