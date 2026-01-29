//! API routes

use crate::{handlers, state::AppState};
use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use randscan_ws::WsState;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

/// WebSocket upgrade handler that works with AppState
async fn app_ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let ws_state = WsState::new(state.ws_manager.clone());
    ws.on_upgrade(move |socket| handle_ws_socket(socket, ws_state))
}

/// Handle WebSocket connection
async fn handle_ws_socket(socket: WebSocket, state: WsState) {
    randscan_ws::handle_socket(socket, state).await;
}

/// Create the API router
pub fn create_router(state: AppState) -> Router {
    let api_routes = Router::new()
        // Health
        .route("/health", get(handlers::health))
        // Blocks
        .route("/blocks", get(handlers::get_blocks))
        .route("/blocks/latest", get(handlers::get_latest_block))
        .route("/blocks/:id", get(handlers::get_block))
        // Transactions
        .route("/transactions", get(handlers::get_transactions))
        .route("/transactions/:id", get(handlers::get_transaction))
        // Accounts
        .route("/accounts/:address", get(handlers::get_account))
        .route(
            "/accounts/:address/transactions",
            get(handlers::get_account_transactions),
        )
        // Validators
        .route("/validators", get(handlers::get_validators))
        .route("/validators/:id", get(handlers::get_validator))
        // Tokens
        .route("/tokens", get(handlers::get_tokens))
        .route("/tokens/:mint", get(handlers::get_token))
        // Stats
        .route("/stats", get(handlers::get_stats))
        .route("/stats/hourly", get(handlers::get_hourly_stats))
        // Search
        .route("/search", get(handlers::search));

    Router::new()
        .nest("/api/v1", api_routes)
        .route("/ws", get(app_ws_handler))
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state)
}
