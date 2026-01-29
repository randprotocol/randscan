//! RandScan API Server

use anyhow::Result;
use randscan_api::{create_router, ApiConfig, AppState};
use randscan_db::{create_pool, run_migrations, DatabaseConfig, DbPool};
use randscan_indexer::{Broadcaster, IndexerConfig, IndexerService};
use randscan_ws::WsManager;
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(true)
        .with_file(true)
        .with_line_number(true)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting RandScan API Server...");

    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize database
    let db_config = DatabaseConfig::from_env();
    info!("Connecting to database: {}", db_config.url);

    let pool = create_pool(&db_config).await?;
    info!("Database connected");

    // Run migrations
    info!("Running database migrations...");
    run_migrations(&pool).await?;
    info!("Migrations complete");

    let db_pool = DbPool::new(pool);

    // Create broadcaster for real-time updates
    let broadcaster = Broadcaster::new();

    // Create WebSocket manager
    let ws_manager = Arc::new(WsManager::new());

    // Start broadcast listener
    let ws_manager_clone = ws_manager.clone();
    let broadcast_receiver = broadcaster.subscribe();
    ws_manager_clone
        .start_broadcast_listener(broadcast_receiver)
        .await;

    // Create indexer service
    let indexer_config = IndexerConfig::from_env();
    let indexer = Arc::new(IndexerService::new(
        indexer_config,
        db_pool.clone(),
        Some(broadcaster),
    ));

    // Start indexer in background
    let indexer_clone = indexer.clone();
    tokio::spawn(async move {
        if let Err(e) = indexer_clone.run().await {
            tracing::error!("Indexer error: {}", e);
        }
    });

    // Create application state
    let state = AppState::new(db_pool, Some(indexer), ws_manager);

    // Create router
    let app = create_router(state);

    // Start server
    let api_config = ApiConfig::from_env();
    info!("Starting API server on {}", api_config.listen_addr);

    let listener = tokio::net::TcpListener::bind(api_config.listen_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
