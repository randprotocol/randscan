//! WebSocket types for RandScan real-time updates

use serde::{Deserialize, Serialize};

/// WebSocket message from client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsClientMessage {
    Subscribe {
        channel: WsChannel,
        #[serde(skip_serializing_if = "Option::is_none")]
        params: Option<WsSubscribeParams>,
    },
    Unsubscribe {
        channel: WsChannel,
        #[serde(skip_serializing_if = "Option::is_none")]
        params: Option<WsSubscribeParams>,
    },
    Ping,
}

/// WebSocket message to client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsServerMessage {
    Subscribed {
        channel: WsChannel,
        subscription_id: String,
    },
    Unsubscribed {
        channel: WsChannel,
    },
    Pong,
    Error {
        message: String,
    },
    // Data messages
    NewBlock {
        block: super::BlockSummary,
    },
    NewTransaction {
        transaction: super::TransactionSummary,
    },
    AccountUpdate {
        account: super::Account,
    },
    ValidatorUpdate {
        validator: super::Validator,
    },
    StatsUpdate {
        stats: super::NetworkStats,
    },
}

/// WebSocket subscription channels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WsChannel {
    Blocks,
    Transactions,
    Account,
    Validator,
    Stats,
}

impl WsChannel {
    pub fn as_str(&self) -> &'static str {
        match self {
            WsChannel::Blocks => "blocks",
            WsChannel::Transactions => "transactions",
            WsChannel::Account => "account",
            WsChannel::Validator => "validator",
            WsChannel::Stats => "stats",
        }
    }
}

/// Subscription parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsSubscribeParams {
    /// For account channel - address to watch
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    /// For validator channel - validator ID to watch
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validator_id: Option<String>,
    /// For transactions channel - filter by type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload_type: Option<String>,
    /// For transactions channel - filter by sender
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender: Option<String>,
}

/// Subscription state
#[derive(Debug, Clone)]
pub struct Subscription {
    pub id: String,
    pub channel: WsChannel,
    pub params: Option<WsSubscribeParams>,
}

impl Subscription {
    pub fn new(channel: WsChannel, params: Option<WsSubscribeParams>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            channel,
            params,
        }
    }

    pub fn matches_block(&self, _block: &super::BlockSummary) -> bool {
        self.channel == WsChannel::Blocks
    }

    pub fn matches_transaction(&self, tx: &super::TransactionSummary) -> bool {
        if self.channel != WsChannel::Transactions {
            return false;
        }
        if let Some(params) = &self.params {
            if let Some(pt) = &params.payload_type {
                if &tx.payload_type != pt {
                    return false;
                }
            }
            if let Some(sender) = &params.sender {
                if &tx.sender != sender {
                    return false;
                }
            }
        }
        true
    }

    pub fn matches_account(&self, account: &super::Account) -> bool {
        if self.channel != WsChannel::Account {
            return false;
        }
        if let Some(params) = &self.params {
            if let Some(addr) = &params.address {
                return &account.address == addr;
            }
        }
        false
    }

    pub fn matches_validator(&self, validator: &super::Validator) -> bool {
        if self.channel != WsChannel::Validator {
            return false;
        }
        if let Some(params) = &self.params {
            if let Some(vid) = &params.validator_id {
                return &validator.validator_id == vid;
            }
        }
        true
    }
}

/// Broadcast event for internal use
#[derive(Debug, Clone)]
pub enum BroadcastEvent {
    NewBlock(super::BlockSummary),
    NewTransaction(super::TransactionSummary),
    AccountUpdate(super::Account),
    ValidatorUpdate(super::Validator),
    StatsUpdate(super::NetworkStats),
}
