use serde::{Deserialize, Serialize};

/// Network statistics: a mix of indexed counts and the node's live status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub chain_id: i64,
    pub symbol: String,
    pub decimals: i16,
    pub height: i64,
    pub view: i64,
    pub total_transactions: i64,
    /// Leaves in the commitment tree: every note the chain has ever created.
    pub notes: i64,
    /// Nullifiers published: every note the chain has ever spent.
    pub nullifiers: i64,
    /// Register entries (every validator that has bonded, active or not).
    pub validator_count: i64,
    /// Validators in the set running the current epoch.
    pub active_validator_count: i64,
    /// Sum of the active set's stake, units.
    pub total_stake: String,
    /// The supply audit's `total_supply` (units), or "0" on a node without `shrugg_getSupply`.
    pub total_supply: String,
    /// The supply audit's `pool_value`: what the notes in the tree are worth in total.
    pub pool_value: Option<String>,
    pub program_count: i64,
    pub avg_block_time_ms: f64,
    pub peer_count: i32,
    pub mempool_size: i32,
    pub node_syncing: bool,
    pub faucet: bool,
    pub confidential: bool,
    pub current_leader: Option<String>,
    /// The commitment tree's current root.
    pub tree_root: Option<String>,
    /// The bundle guest every proof on this chain is checked against.
    pub hc_bundle: Option<String>,
    pub epoch: Option<i64>,
    pub epoch_blocks: Option<i64>,
    pub updated_at: String,
}

/// The node's supply audit (`shrugg_getSupply`, phase S2): every crossing of the pool boundary is
/// public, so these are exact. All amounts are unit strings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Supply {
    pub height: i64,
    pub genesis_deposited: String,
    pub genesis_staked: String,
    pub faucet_minted: String,
    pub withdraw_deposited: String,
    pub fees_paid: String,
    pub burned: String,
    pub pool_value: String,
    pub register_total: String,
    pub total_supply: String,
    pub invariant_holds: bool,
}
