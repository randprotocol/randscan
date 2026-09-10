use crate::{state::AppState, ApiResult};
use axum::{extract::State, Json};
use randscan_core::NodeInfo;

/// GET /api/v1/nodes
pub async fn list_nodes(State(state): State<AppState>) -> ApiResult<Json<Vec<NodeInfo>>> {
    Ok(Json(state.indexer.nodes().await))
}
