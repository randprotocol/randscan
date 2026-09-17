use crate::{handlers, state::AppState};
use axum::{
    extract::{ws::WebSocketUpgrade, State},
    middleware,
    response::IntoResponse,
    routing::{delete, get, post},
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
    let auth_public = Router::new()
        .route("/auth/signup", post(handlers::signup))
        .route("/auth/login", post(handlers::login))
        .route("/auth/forgot", post(handlers::forgot_password))
        .route("/auth/reset", post(handlers::reset_password))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            crate::ratelimit::auth_limit,
        ));

    let api = Router::new()
        .route("/health", get(handlers::health))
        .route("/stats", get(handlers::get_stats))
        .route("/blocks", get(handlers::list_blocks))
        .route("/blocks/latest", get(handlers::latest_blocks))
        .route("/blocks/:id", get(handlers::get_block))
        .route("/transactions", get(handlers::list_transactions))
        .route("/transactions/latest", get(handlers::latest_transactions))
        .route("/transactions/:hash", get(handlers::get_transaction))
        .route(
            "/transactions/:hash/envelopes",
            get(handlers::transaction_envelopes),
        )
        .route("/envelopes", get(handlers::list_envelopes))
        .route("/accounts/:address", get(handlers::no_accounts))
        .route(
            "/accounts/:address/transactions",
            get(handlers::no_accounts),
        )
        .route("/notes", get(handlers::list_notes))
        .route("/notes/:id", get(handlers::get_note))
        .route("/nullifiers/:nf", get(handlers::get_nullifier))
        .route("/bridge", get(handlers::get_bridge))
        .route("/bridge/assets", get(handlers::bridge_assets))
        .route("/bridge/tokens", get(handlers::bridge_tokens))
        .route("/supply", get(handlers::get_supply))
        .route("/validators", get(handlers::list_validators))
        .route("/validators/:address", get(handlers::get_validator))
        .route("/programs", get(handlers::list_programs))
        .route("/programs/:id", get(handlers::get_program))
        .route("/nodes", get(handlers::list_nodes))
        .route("/search", get(handlers::search))
        .route("/auth/logout", post(handlers::logout))
        .route("/auth/me", get(handlers::me))
        .route("/auth/password", post(handlers::change_password))
        .route("/keys", get(handlers::list_keys).post(handlers::create_key))
        .route("/keys/:id", delete(handlers::revoke_key))
        .merge(auth_public)
        .layer(middleware::from_fn_with_state(
            state.clone(),
            crate::ratelimit::api_limit,
        ));

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
