use crate::{handlers, state::AppState};
use axum::{
    extract::{ws::WebSocketUpgrade, State},
    response::IntoResponse,
    routing::get,
    Router,
};
use randscan_ws::WsState;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

async fn ws_upgrade(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    let ws_state = WsState::new(state.ws_manager.clone());
    ws.on_upgrade(move |socket| randscan_ws::handle_socket(socket, ws_state))
}

pub fn create_router(state: AppState) -> Router {
    let api = Router::new()
        .route("/health", get(handlers::health))
        .route("/stats", get(handlers::get_stats))
        .route("/blocks", get(handlers::list_blocks))
        .route("/blocks/latest", get(handlers::latest_blocks))
        .route("/blocks/:id", get(handlers::get_block))
        .route("/transactions", get(handlers::list_transactions))
        .route("/transactions/latest", get(handlers::latest_transactions))
        .route("/transactions/:hash", get(handlers::get_transaction))
        .route("/accounts/:address", get(handlers::get_account))
        .route(
            "/accounts/:address/transactions",
            get(handlers::account_transactions),
        )
        .route("/validators", get(handlers::list_validators))
        .route("/validators/:address", get(handlers::get_validator))
        .route("/programs", get(handlers::list_programs))
        .route("/programs/:id", get(handlers::get_program))
        .route("/nodes", get(handlers::list_nodes))
        .route("/search", get(handlers::search));

    Router::new()
        .nest("/api/v1", api)
        .route("/ws", get(ws_upgrade))
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state)
}
