use crate::{error::AppError, state::AppState, ApiResult};
use axum::{extract::State, Json};
use randscan_core::{approved_token, ApprovedToken, BridgeAssetActivity, BridgeState, Supply, APPROVED_TOKENS};

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

/// GET /api/v1/bridge/assets — every registered bridged asset with what the chain has seen of it.
/// The registry comes from the node, the flows from the indexed transactions; an asset with no
/// flow yet lists zeros, and a flow whose asset the node has not (yet) registered is skipped.
/// Empty on a chain without a bridge.
pub async fn bridge_assets(State(state): State<AppState>) -> ApiResult<Json<Vec<BridgeAssetActivity>>> {
    let Some(bridge) = state.indexer.bridge().await.filter(|b| b.enabled) else {
        return Ok(Json(vec![]));
    };
    let flows = randscan_db::bridge_asset_flows(state.db.inner()).await?;
    let out = bridge
        .assets
        .iter()
        .map(|a| {
            let f = flows.iter().find(|f| f.asset_index == a.index);
            let known = approved_token(a.chain, &a.token);
            let deposited = f.map(|f| f.deposited.clone()).unwrap_or_else(|| "0".into());
            let burned = f.map(|f| f.burned.clone()).unwrap_or_else(|| "0".into());
            BridgeAssetActivity {
                index: a.index,
                chain: a.chain,
                token: a.token.clone(),
                asset_id: a.asset_id.clone(),
                symbol: known.map(|k| k.symbol.to_string()),
                name: known.map(|k| k.name.to_string()),
                decimals: known.map(|k| k.decimals),
                deposits: f.map_or(0, |f| f.deposits),
                deposited: deposited.clone(),
                burns: f.map_or(0, |f| f.burns),
                burned: burned.clone(),
                outstanding: units_sub(&deposited, &burned),
                first_height: f.and_then(|f| f.first_height),
                last_height: f.and_then(|f| f.last_height),
            }
        })
        .collect();
    Ok(Json(out))
}

/// GET /api/v1/bridge/tokens — the tokens the bridge accepts, with each contract address as the
/// chain prints it and as the registry stores it. A static list; it does not depend on the node.
pub async fn bridge_tokens() -> Json<&'static [ApprovedToken]> {
    Json(APPROVED_TOKENS)
}

/// `a - b` on decimal unit strings, saturating at zero (the node never lets burns exceed
/// deposits, but a partially indexed chain can see a burn before its deposit).
fn units_sub(a: &str, b: &str) -> String {
    let a: u128 = a.parse().unwrap_or(0);
    let b: u128 = b.parse().unwrap_or(0);
    a.saturating_sub(b).to_string()
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
