//! RandScan API server: runs the indexer and serves the REST + WebSocket API.

use anyhow::Result;
use randscan_api::{create_router, ApiConfig, AppState};
use randscan_db::{
    create_pool, delete_all_expired_sessions, delete_expired_password_resets, run_migrations,
    DatabaseConfig, DbPool,
};
use randscan_indexer::{Broadcaster, IndexerConfig, IndexerService};
use randscan_ws::WsManager;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    if std::env::args().nth(1).as_deref() == Some("hash-password") {
        return hash_password_cli().await;
    }

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

    let api_config = Arc::new(ApiConfig::from_env());
    // The Resend key is read here and handed straight to the mailer; it never lives in
    // `ApiConfig` (which derives Debug) or in logs.
    let mailer: Option<Arc<dyn randscan_api::mail::MailSender>> =
        match std::env::var("RESEND_API_KEY")
            .ok()
            .map(|k| k.trim().to_string())
            .filter(|k| !k.is_empty())
        {
            Some(key) => {
                info!(
                    "password reset enabled via Resend, from {}",
                    api_config.mail_from
                );
                Some(Arc::new(randscan_api::mail::ResendMailer::new(
                    key,
                    api_config.mail_from.clone(),
                )))
            }
            None => {
                info!("RESEND_API_KEY unset: password reset disabled");
                None
            }
        };
    let state = AppState {
        db: db_pool,
        indexer,
        ws_manager,
        config: api_config.clone(),
        limiter: Arc::new(randscan_api::ratelimit::RateLimiter::new()),
        mailer,
    };
    state.limiter.clone().spawn_sweeper();

    // Compute the dummy password hash now, on a blocking-pool thread, so the first failed
    // login (unknown email) does not pay for it inline and block a runtime worker thread.
    tokio::task::spawn_blocking(|| {
        randscan_api::auth::dummy_hash();
    });

    // Session reaper: delete expired sessions once at startup, then every hour.
    let reaper_db = state.db.clone();
    tokio::spawn(async move {
        loop {
            match delete_all_expired_sessions(reaper_db.inner()).await {
                Ok(n) => tracing::debug!("session reaper: deleted {} expired session(s)", n),
                Err(e) => tracing::warn!("session reaper: {}", e),
            }
            match delete_expired_password_resets(reaper_db.inner()).await {
                Ok(n) => tracing::debug!("reset reaper: deleted {} stale reset token(s)", n),
                Err(e) => tracing::warn!("reset reaper: {}", e),
            }
            tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
        }
    });

    let app = create_router(state);

    info!("listening on {}", api_config.listen_addr);
    let listener = tokio::net::TcpListener::bind(api_config.listen_addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await?;
    Ok(())
}

/// `randscan-api hash-password` reads one line from stdin and prints the argon2id PHC string.
/// Operators use it with the password-reset runbook in the README.
async fn hash_password_cli() -> Result<()> {
    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;
    let password = line.trim_end_matches(['\r', '\n']).to_string();
    randscan_api::auth::validate_password(&password).map_err(|e| anyhow::anyhow!(e))?;
    let hash = randscan_api::auth::hash_password(password)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    println!("{}", hash);
    Ok(())
}
