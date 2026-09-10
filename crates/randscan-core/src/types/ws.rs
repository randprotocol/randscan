//! WebSocket protocol.

use serde::{Deserialize, Serialize};

use super::{BlockSummary, NetworkStats, TransactionSummary};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsClientMessage {
    Subscribe { channel: WsChannel },
    Unsubscribe { channel: WsChannel },
    Ping,
}

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
    NewBlock {
        block: BlockSummary,
    },
    NewTransaction {
        transaction: TransactionSummary,
    },
    StatsUpdate {
        stats: NetworkStats,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WsChannel {
    Blocks,
    Transactions,
    Stats,
}

/// Event produced by the indexer and fanned out to subscribers.
#[derive(Debug, Clone)]
pub enum BroadcastEvent {
    NewBlock(BlockSummary),
    NewTransaction(TransactionSummary),
    StatsUpdate(NetworkStats),
}

impl BroadcastEvent {
    pub fn channel(&self) -> WsChannel {
        match self {
            BroadcastEvent::NewBlock(_) => WsChannel::Blocks,
            BroadcastEvent::NewTransaction(_) => WsChannel::Transactions,
            BroadcastEvent::StatsUpdate(_) => WsChannel::Stats,
        }
    }

    pub fn to_message(&self) -> WsServerMessage {
        match self {
            BroadcastEvent::NewBlock(b) => WsServerMessage::NewBlock { block: b.clone() },
            BroadcastEvent::NewTransaction(t) => WsServerMessage::NewTransaction {
                transaction: t.clone(),
            },
            BroadcastEvent::StatsUpdate(s) => WsServerMessage::StatsUpdate { stats: s.clone() },
        }
    }
}
