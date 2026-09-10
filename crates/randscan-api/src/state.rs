use randscan_db::DbPool;
use randscan_indexer::IndexerService;
use randscan_ws::WsManager;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub indexer: Arc<IndexerService>,
    pub ws_manager: Arc<WsManager>,
}
