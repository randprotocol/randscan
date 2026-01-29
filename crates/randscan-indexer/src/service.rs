//! Indexer service - main sync loop

use crate::{BlockProcessor, Broadcaster, IndexerConfig, RpcClient};
use randscan_core::NetworkStats;
use randscan_db::{self as db, DbPool};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn, debug};

/// Indexer state
#[derive(Debug, Clone)]
pub struct IndexerState {
    pub current_height: u64,
    pub node_height: u64,
    pub is_syncing: bool,
    pub is_connected: bool,
}

/// Main indexer service
pub struct IndexerService {
    config: IndexerConfig,
    rpc: RpcClient,
    processor: BlockProcessor,
    pool: DbPool,
    broadcaster: Option<Broadcaster>,
    state: Arc<RwLock<IndexerState>>,
}

impl IndexerService {
    pub fn new(
        config: IndexerConfig,
        pool: DbPool,
        broadcaster: Option<Broadcaster>,
    ) -> Self {
        let rpc = RpcClient::new(&config.rpc_url);
        let processor = BlockProcessor::new(pool.clone(), broadcaster.clone());

        Self {
            config,
            rpc,
            processor,
            pool,
            broadcaster,
            state: Arc::new(RwLock::new(IndexerState {
                current_height: 0,
                node_height: 0,
                is_syncing: false,
                is_connected: false,
            })),
        }
    }

    /// Get current indexer state
    pub async fn get_state(&self) -> IndexerState {
        self.state.read().await.clone()
    }

    /// Start the indexer service
    pub async fn run(&self) -> Result<()> {
        info!("Starting indexer service...");

        // Initialize native tokens
        db::initialize_native_tokens(self.pool.inner()).await?;

        // Get last indexed height from database
        let last_height = db::get_last_indexed_height(self.pool.inner()).await?;

        {
            let mut state = self.state.write().await;
            state.current_height = last_height as u64;
        }

        info!("Resuming from height {}", last_height);

        // Main sync loop
        loop {
            if let Err(e) = self.sync_loop().await {
                error!("Sync loop error: {}", e);
                sleep(Duration::from_secs(5)).await;
            }
        }
    }

    /// Main synchronization loop
    async fn sync_loop(&self) -> Result<()> {
        // Check connection
        let connected = self.rpc.is_connected().await;
        {
            let mut state = self.state.write().await;
            state.is_connected = connected;
        }

        if !connected {
            warn!("Cannot connect to RPC node at {}", self.config.rpc_url);
            sleep(Duration::from_secs(5)).await;
            return Ok(());
        }

        // Get current node height
        let node_height = self.rpc.get_block_height().await?;
        let current_height = {
            let state = self.state.read().await;
            state.current_height
        };

        {
            let mut state = self.state.write().await;
            state.node_height = node_height;
        }

        // Check if we need to sync
        if current_height >= node_height {
            // Up to date, just poll for new blocks
            sleep(self.config.poll_interval).await;
            return Ok(());
        }

        // Calculate how many blocks to sync
        let blocks_behind = node_height - current_height;
        let is_initial_sync = blocks_behind > self.config.batch_size;

        {
            let mut state = self.state.write().await;
            state.is_syncing = is_initial_sync;
        }

        if is_initial_sync {
            info!(
                "Initial sync: {} blocks behind (height {} -> {})",
                blocks_behind, current_height, node_height
            );
            db::set_syncing(self.pool.inner(), true).await?;
        }

        // Sync blocks in batches
        let batch_end = (current_height + self.config.batch_size).min(node_height);

        for height in (current_height + 1)..=batch_end {
            match self.rpc.get_block(height).await {
                Ok(block) => {
                    self.processor.process_block(block).await?;

                    // Update checkpoint
                    db::update_last_indexed(
                        self.pool.inner(),
                        height as i64,
                        "",
                    ).await?;

                    {
                        let mut state = self.state.write().await;
                        state.current_height = height;
                    }
                }
                Err(e) => {
                    warn!("Failed to fetch block {}: {}", height, e);
                    // Skip and continue
                }
            }

            // Log progress during initial sync
            if is_initial_sync && height % 100 == 0 {
                info!("Synced to height {} / {}", height, node_height);
            }
        }

        // Mark finalized blocks
        if node_height > self.config.finality_depth {
            let finalize_height = node_height - self.config.finality_depth;
            self.processor.finalize_blocks(finalize_height).await?;
            db::update_last_finalized(self.pool.inner(), finalize_height as i64).await?;
        }

        // Update network stats periodically
        if !is_initial_sync {
            self.update_stats().await?;
        }

        if is_initial_sync && batch_end >= node_height {
            info!("Initial sync complete!");
            db::set_syncing(self.pool.inner(), false).await?;
        }

        Ok(())
    }

    /// Update network statistics
    async fn update_stats(&self) -> Result<()> {
        let db = self.pool.inner();

        // Get counts
        let block_height = db::get_last_indexed_height(db).await?;
        let total_transactions = db::count_transactions(db, None, None, None, None).await?;
        let total_accounts = db::count_accounts(db).await?;
        let total_validators = db::count_validators(db, None).await?;
        let active_validators = db::count_validators(db, Some(true)).await?;
        let total_staked = db::get_total_staked(db).await?;

        // Get token supplies from RPC
        let (atlas_supply, shrug_supply, shrug_burned) = match (
            self.rpc.get_token_supply("ATLAS").await,
            self.rpc.get_token_supply("SHRUG").await,
        ) {
            (Ok(atlas), Ok(shrug)) => (
                atlas.total_supply as i64,
                shrug.total_supply as i64,
                shrug.burned as i64,
            ),
            _ => (0, 0, 0),
        };

        // Get epoch info
        let current_epoch = match self.rpc.get_epoch_info().await {
            Ok(info) => info.epoch as i64,
            Err(_) => 0,
        };

        // Calculate TPS (simplified - last 10 blocks)
        let tps_current = 0.0; // Would need to calculate from recent blocks

        db::update_network_stats(
            db,
            block_height,
            total_transactions,
            total_accounts,
            total_validators,
            active_validators,
            atlas_supply,
            total_staked,
            shrug_supply,
            shrug_burned,
            0.0, // avg_block_time
            tps_current,
            tps_current,
            current_epoch,
        )
        .await?;

        // Broadcast stats update
        if let Some(ref broadcaster) = self.broadcaster {
            let stats = NetworkStats {
                block_height,
                total_transactions,
                total_accounts,
                total_validators,
                active_validators,
                atlas_total_supply: atlas_supply,
                atlas_staked: total_staked,
                shrug_total_supply: shrug_supply,
                shrug_burned,
                avg_block_time: 0.0,
                tps_current,
                tps_peak: 0.0,
                current_epoch,
                updated_at: chrono::Utc::now().timestamp_millis(),
            };
            broadcaster.broadcast_stats_update(stats);
        }

        debug!("Updated network stats");
        Ok(())
    }

    /// Sync validators from node
    pub async fn sync_validators(&self) -> Result<()> {
        let validators = self.rpc.get_validators().await?;
        let count = validators.len();

        for val in validators {
            self.processor.update_validator(&val).await?;
        }

        info!("Synced {} validators", count);
        Ok(())
    }

    /// Get broadcaster reference
    pub fn broadcaster(&self) -> Option<&Broadcaster> {
        self.broadcaster.as_ref()
    }
}
