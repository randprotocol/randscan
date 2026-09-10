use serde::{Deserialize, Serialize};

use super::TransactionSummary;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramSummary {
    pub id: String,
    pub deployer: String,
    pub deploy_tx: String,
    pub deployed_at_height: i64,
    pub base_pc: i64,
    pub words_len: i64,
    pub code_hash: String,
    pub call_count: i64,
    pub last_called_height: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramDetail {
    #[serde(flatten)]
    pub program: ProgramSummary,
    pub recent_calls: Vec<TransactionSummary>,
}
