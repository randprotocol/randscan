//! Application state

use randscan_db::DbPool;
use randscan_indexer::IndexerService;
use randscan_ws::WsManager;
use std::sync::Arc;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub indexer: Option<Arc<IndexerService>>,
    pub ws_manager: Arc<WsManager>,
}

impl AppState {
    pub fn new(
        db: DbPool,
        indexer: Option<Arc<IndexerService>>,
        ws_manager: Arc<WsManager>,
    ) -> Self {
        Self {
            db,
            indexer,
            ws_manager,
        }
    }
}
