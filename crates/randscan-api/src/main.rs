//! RandScan API server: runs the indexer and serves the REST + WebSocket API.

use anyhow::Result;
use randscan_api::{create_router, ApiConfig, AppState};
use randscan_db::{create_pool, run_migrations, DatabaseConfig, DbPool};
use randscan_indexer::{Broadcaster, IndexerConfig, IndexerService};
use randscan_ws::WsManager;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    info!("RandScan API {} starting", env!("CARGO_PKG_VERSION"));

    let db_config = DatabaseConfig::from_env();
    let pool = create_pool(&db_config).await?;
    run_migrations(&pool).await?;
    let db_pool = DbPool::new(pool);

    let broadcaster = Broadcaster::new();
    let ws_manager = Arc::new(WsManager::new());
    ws_manager
        .clone()
        .start_broadcast_listener(broadcaster.subscribe())
        .await;

    let indexer = Arc::new(IndexerService::new(
        IndexerConfig::from_env(),
        db_pool.clone(),
        broadcaster,
    ));
    let indexer_task = indexer.clone();
    tokio::spawn(async move {
        if let Err(e) = indexer_task.run().await {
            tracing::error!("indexer stopped: {:#}", e);
        }
    });

    let state = AppState {
        db: db_pool,
        indexer,
        ws_manager,
    };
    let app = create_router(state);

    let api_config = ApiConfig::from_env();
    info!("listening on {}", api_config.listen_addr);
    let listener = tokio::net::TcpListener::bind(api_config.listen_addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
