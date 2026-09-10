//! Sync loop: follow the node head, index new blocks, refresh validators and stats.

use crate::{BlockProcessor, Broadcaster, IndexerConfig, NodeTracker, RpcClient};
use anyhow::Result;
use randscan_core::NetworkStats;
use randscan_core::NodeInfo;
use randscan_db::{self as db, DbPool, StatsUpdate};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

#[derive(Debug, Clone, Default)]
pub struct IndexerState {
    /// Highest indexed height (-1 if nothing indexed).
    pub current_height: i64,
    pub node_height: i64,
    pub is_syncing: bool,
    pub is_connected: bool,
}

pub struct IndexerService {
    config: IndexerConfig,
    rpc: RpcClient,
    processor: BlockProcessor,
    pool: DbPool,
    broadcaster: Broadcaster,
    state: Arc<RwLock<IndexerState>>,
    chain: RwLock<Option<(i64, String, i16)>>,
    last_stats: RwLock<Option<Instant>>,
    nodes: Arc<NodeTracker>,
}

impl IndexerService {
    pub fn new(config: IndexerConfig, pool: DbPool, broadcaster: Broadcaster) -> Self {
        let rpc = RpcClient::new(&config.rpc_url);
        let processor = BlockProcessor::new(pool.clone(), rpc.clone());
        let nodes = Arc::new(NodeTracker::new(
            rpc.clone(),
            pool.clone(),
            config.nodes_interval,
        ));
        Self {
            config,
            rpc,
            processor,
            pool,
            broadcaster,
            state: Arc::new(RwLock::new(IndexerState {
                current_height: -1,
                ..Default::default()
            })),
            chain: RwLock::new(None),
            last_stats: RwLock::new(None),
            nodes,
        }
    }

    pub fn rpc(&self) -> &RpcClient {
        &self.rpc
    }

    /// Current view of this node and its peers (nodes map).
    pub async fn nodes(&self) -> Vec<NodeInfo> {
        self.nodes.nodes().await
    }

    pub async fn get_state(&self) -> IndexerState {
        self.state.read().await.clone()
    }

    pub async fn run(&self) -> Result<()> {
        info!("indexer starting against {}", self.config.rpc_url);
        let st = db::get_indexer_state(self.pool.inner()).await?;
        self.state.write().await.current_height = st.next_height - 1;
        info!("resuming from height {}", st.next_height);

        let tracker = self.nodes.clone();
        tokio::spawn(async move { tracker.run().await });

        loop {
            match self.sync_once().await {
                Ok(idle) => {
                    if idle {
                        sleep(self.config.poll_interval).await;
                    }
                }
                Err(e) => {
                    error!("sync error: {:#}", e);
                    self.state.write().await.is_connected = false;
                    sleep(Duration::from_secs(3)).await;
                }
            }
        }
    }

    /// One pass. Returns true when caught up (caller sleeps).
    async fn sync_once(&self) -> Result<bool> {
        let head = self.rpc.head().await?;
        let node_height = head.height as i64;
        {
            let mut s = self.state.write().await;
            s.is_connected = true;
            s.node_height = node_height;
        }

        let next = db::get_indexer_state(self.pool.inner()).await?.next_height;
        let behind = node_height - next + 1;
        let catching_up = behind > 5;
        {
            let mut s = self.state.write().await;
            if s.is_syncing != catching_up {
                s.is_syncing = catching_up;
                let _ = db::set_syncing(self.pool.inner(), catching_up).await;
                if catching_up {
                    info!(
                        "catching up: {} blocks behind ({} -> {})",
                        behind, next, node_height
                    );
                }
            }
        }
        if next > node_height {
            self.maybe_refresh_stats(false).await;
            return Ok(true);
        }

        let end = (next + self.config.batch_size as i64 - 1).min(node_height);
        let mut h = next;
        while h <= end {
            let block = match self.rpc.block_by_height(h as u64).await? {
                Some(b) => b,
                None => {
                    warn!("block {} not served yet", h);
                    return Ok(true);
                }
            };

            if h > 0 {
                let stored = db::get_block_hash_at(self.pool.inner(), h - 1).await?;
                match stored {
                    Some(ref parent) if parent == &block.parent => {}
                    Some(parent) => {
                        warn!(
                            "parent mismatch at {}: stored {} vs node {}",
                            h, parent, block.parent
                        );
                        self.processor.rewind_to(h - 1).await?;
                        return Ok(false);
                    }
                    None => {
                        warn!("missing block {} in database; rewinding", h - 1);
                        self.processor.rewind_to(h - 1).await?;
                        return Ok(false);
                    }
                }
            }

            let processed = self.processor.process_block(block).await?;
            self.state.write().await.current_height = h;
            self.broadcaster.block(processed.block);
            for tx in processed.transactions {
                self.broadcaster.transaction(tx);
            }
            if catching_up && h % 500 == 0 {
                info!("indexed height {} / {}", h, node_height);
            }
            h += 1;
        }

        if end >= node_height {
            if catching_up {
                info!("caught up at height {}", node_height);
            }
            self.maybe_refresh_stats(true).await;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn maybe_refresh_stats(&self, force: bool) {
        let due = {
            let last = self.last_stats.read().await;
            force
                || last
                    .map(|t| t.elapsed() >= self.config.stats_interval)
                    .unwrap_or(true)
        };
        if !due {
            return;
        }
        *self.last_stats.write().await = Some(Instant::now());
        if let Err(e) = self.refresh_validators().await {
            warn!("validator refresh failed: {:#}", e);
        }
        match self.refresh_stats().await {
            Ok(stats) => self.broadcaster.stats(stats),
            Err(e) => warn!("stats refresh failed: {:#}", e),
        }
    }

    async fn refresh_validators(&self) -> Result<()> {
        let set = self.rpc.validators().await?;
        let pairs: Vec<(String, String)> = set
            .iter()
            .map(|v| (v.address.clone(), v.stake.clone()))
            .collect();
        let mut conn = self.pool.inner().acquire().await?;
        db::replace_validators(&mut conn, &pairs).await?;
        drop(conn);
        // Genesis allocations are not transactions: make sure validator accounts exist.
        for v in &set {
            if db::get_account(self.pool.inner(), &v.address)
                .await?
                .is_none()
            {
                self.processor.refresh_account(&v.address, 0).await;
            }
        }
        Ok(())
    }

    async fn chain_info(&self) -> Result<(i64, String, i16)> {
        if let Some(c) = self.chain.read().await.clone() {
            return Ok(c);
        }
        let id = self.rpc.chain_id().await? as i64;
        let token = self.rpc.token_info().await?;
        let c = (id, token.symbol, token.decimals as i16);
        *self.chain.write().await = Some(c.clone());
        Ok(c)
    }

    async fn refresh_stats(&self) -> Result<NetworkStats> {
        let pool = self.pool.inner();
        let status = self.rpc.status().await?;
        let (chain_id, symbol, decimals) = self.chain_info().await?;
        let height = db::max_block_height(pool).await?.unwrap_or(-1).max(0);
        let total_transactions = db::count_transactions(pool, None, None, None).await?;
        let total_accounts = db::count_accounts(pool).await?;
        let validators = db::list_validators(pool).await?;
        let validator_count = validators.len() as i64;
        let total_stake = db::total_stake(pool).await?;
        let total_supply = db::total_supply(pool).await?;
        let program_count = db::count_programs(pool).await?;
        let avg_block_time_ms = db::avg_block_time_ms(pool, 100).await?;
        let current_leader = if validators.is_empty() {
            None
        } else {
            validators
                .get((status.view % validators.len() as u64) as usize)
                .map(|v| v.address.clone())
        };

        db::update_network_stats(
            pool,
            &StatsUpdate {
                chain_id,
                symbol: &symbol,
                decimals,
                height,
                view: status.view as i64,
                total_transactions,
                total_accounts,
                validator_count,
                total_stake: &total_stake,
                total_supply: &total_supply,
                program_count,
                avg_block_time_ms,
                peer_count: status.peer_count as i32,
                mempool_size: status.mempool_size as i32,
                node_syncing: status.syncing,
                faucet: status.faucet,
                confidential: status.confidential,
                current_leader: current_leader.as_deref(),
            },
        )
        .await?;

        Ok(db::get_network_stats(pool).await?.into())
    }
}
