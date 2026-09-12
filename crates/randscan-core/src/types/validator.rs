use serde::{Deserialize, Serialize};

use super::BlockSummary;

/// One unbonding entry of a validator: `amount` (units) is released at `release_epoch`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingStake {
    pub release_epoch: i64,
    pub amount: String,
}

/// An entry of the public validator register (shielded pool spec §8): the one place the chain
/// stores amounts in the clear.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    pub address: String,
    /// Stake weight, units of SHRUGG as a decimal string.
    pub stake: String,
    /// Bundle fees credited to this validator as proposer, not yet withdrawn (units).
    pub rewards: String,
    /// The unbonding queue, oldest first.
    pub pending: Vec<PendingStake>,
    /// The shielded address a `withdraw` pays to; `null` on a node that does not serve it.
    pub payout: Option<String>,
    /// What the validator's next signed unbond / withdraw must carry.
    pub nonce: i64,
    /// In the set running the current epoch (what leader rotation runs over).
    pub active: bool,
    /// Share of the active set's stake.
    pub share_percent: f64,
    pub blocks_proposed: i64,
    pub last_proposed_height: Option<i64>,
    pub last_proposed_timestamp_ms: Option<i64>,
    pub sort_index: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorDetail {
    #[serde(flatten)]
    pub validator: Validator,
    pub recent_blocks: Vec<BlockSummary>,
}
