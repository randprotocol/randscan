//! Turns an RPC block into database rows and refreshes the touched accounts.

use crate::rpc::{RpcBlock, RpcClient, RpcTxKind};
use anyhow::{Context, Result};
use randscan_core::{BlockSummary, TransactionSummary, TxKind};
use randscan_db::{self as db, DbPool, NewBlock, NewTx};
use std::collections::{BTreeSet, HashMap};
use tracing::{debug, warn};

pub struct BlockProcessor {
    pool: DbPool,
    rpc: RpcClient,
}

pub struct ProcessedBlock {
    pub block: BlockSummary,
    pub transactions: Vec<TransactionSummary>,
}

struct PreparedReceipt {
    tx_hash: String,
    program: String,
    tier: i32,
    outputs: Vec<i64>,
    effect_to: Option<String>,
    effect_amount: Option<String>,
    height: i64,
    index: i32,
}

impl BlockProcessor {
    pub fn new(pool: DbPool, rpc: RpcClient) -> Self {
        Self { pool, rpc }
    }

    /// Store one block and everything derived from it, then advance `indexer_state.next_height`.
    pub async fn process_block(&self, block: RpcBlock) -> Result<ProcessedBlock> {
        let height = block.height as i64;

        // Receipts come from the node; fetch them before opening the DB transaction.
        let mut receipts = Vec::new();
        for tx in &block.transactions {
            if let RpcTxKind::Call { .. } = tx.kind {
                match self.rpc.receipt(&tx.hash).await {
                    Ok(Some(r)) => receipts.push(PreparedReceipt {
                        tx_hash: tx.hash.clone(),
                        program: r.program,
                        tier: r.tier as i32,
                        outputs: r.outputs,
                        effect_to: r.effect.as_ref().map(|e| e.to.clone()),
                        effect_amount: r.effect.as_ref().map(|e| e.amount.clone()),
                        height: r.height as i64,
                        index: r.index as i32,
                    }),
                    Ok(None) => warn!("no receipt yet for call {} at height {}", tx.hash, height),
                    Err(e) => warn!("receipt fetch failed for {}: {}", tx.hash, e),
                }
            }
        }

        // Program code hashes come from the node (after the zkVM fork they are a Poseidon2 digest,
        // distinct from the content id); fall back to the id if the record is not served yet.
        let mut code_hashes: HashMap<String, String> = HashMap::new();
        for tx in &block.transactions {
            if let RpcTxKind::Deploy { program, .. } = &tx.kind {
                let hash = match self.rpc.program(program).await {
                    Ok(Some(p)) => p.code_hash,
                    Ok(None) => program.clone(),
                    Err(e) => {
                        warn!("program fetch {} failed: {}", program, e);
                        program.clone()
                    }
                };
                code_hashes.insert(program.clone(), hash);
            }
        }

        let mut touched: BTreeSet<String> = BTreeSet::new();
        let mut dbtx = self.pool.inner().begin().await?;

        db::insert_block(
            &mut dbtx,
            &NewBlock {
                hash: &block.hash,
                height,
                view: block.view as i64,
                parent: &block.parent,
                proposer: &block.proposer,
                timestamp_ms: block.timestamp_ms as i64,
                tx_root: &block.tx_root,
                state_root: &block.state_root,
                justify_view: block.justify_view as i64,
                tx_count: block.transactions.len() as i32,
            },
        )
        .await
        .with_context(|| format!("insert block {}", height))?;

        let mut summaries = Vec::with_capacity(block.transactions.len());

        for (i, tx) in block.transactions.iter().enumerate() {
            let tx_index = i as i32;
            let f = TxFields::from_rpc(&tx.kind);
            let kind = f.kind;
            let to = f.to;
            let amount = f.amount;
            let program = f.program;

            db::insert_transaction(
                &mut dbtx,
                &NewTx {
                    hash: &tx.hash,
                    block_hash: &block.hash,
                    height,
                    tx_index,
                    sender: &tx.from,
                    nonce: tx.nonce as i64,
                    fee: &tx.fee,
                    kind: f.kind_tag,
                    chain_id: tx.chain_id as i64,
                    timestamp_ms: block.timestamp_ms as i64,
                    to_address: to,
                    amount,
                    program_id: program,
                    base_pc: f.base_pc,
                    words_len: f.words_len,
                    proof_len: f.proof_len,
                    recipients: &f.recipients,
                    asset: f.asset,
                    bridge_amount: f.bridge_amount,
                    to_chain: f.to_chain,
                    bridge_to: f.bridge_to,
                    bridge_fee: f.bridge_fee,
                    attestation: f.attestation,
                },
            )
            .await
            .with_context(|| format!("insert tx {}", tx.hash))?;

            db::insert_account_transaction(
                &mut dbtx, &tx.from, &tx.hash, "sender", height, tx_index,
            )
            .await?;
            touched.insert(tx.from.clone());
            if let Some(to) = to {
                db::insert_account_transaction(
                    &mut dbtx,
                    to,
                    &tx.hash,
                    "recipient",
                    height,
                    tx_index,
                )
                .await?;
                touched.insert(to.to_string());
            }

            if let RpcTxKind::Deploy {
                base_pc,
                words_len,
                program,
            } = &tx.kind
            {
                db::insert_program(
                    &mut dbtx,
                    program,
                    &tx.from,
                    &tx.hash,
                    height,
                    *base_pc as i64,
                    *words_len as i64,
                    code_hashes.get(program).unwrap_or(program),
                )
                .await?;
            }

            summaries.push(TransactionSummary {
                hash: tx.hash.clone(),
                height,
                block_hash: block.hash.clone(),
                tx_index,
                sender: tx.from.clone(),
                nonce: tx.nonce as i64,
                fee: tx.fee.clone(),
                kind,
                timestamp_ms: block.timestamp_ms as i64,
                to: to.map(str::to_string),
                amount: amount.map(str::to_string),
                program: program.map(str::to_string),
            });
        }

        for r in &receipts {
            db::insert_receipt(
                &mut dbtx,
                &r.tx_hash,
                &r.program,
                r.tier,
                &r.outputs,
                r.effect_to.as_deref(),
                r.effect_amount.as_deref(),
                r.height,
                r.index,
            )
            .await?;
            if let Some(to) = &r.effect_to {
                db::insert_account_transaction(
                    &mut dbtx,
                    to,
                    &r.tx_hash,
                    "recipient",
                    r.height,
                    r.index,
                )
                .await?;
                touched.insert(to.clone());
            }
        }

        // Fees go to the proposer, so its balance moved if the block had transactions.
        if !block.transactions.is_empty() {
            touched.insert(block.proposer.clone());
        }

        db::set_next_height(&mut dbtx, height + 1, Some(&block.hash)).await?;
        dbtx.commit().await?;

        // Balances and nonces are read back from the node rather than re-deriving ledger rules.
        for address in &touched {
            self.refresh_account(address, height).await;
        }

        debug!(
            "indexed block {} ({} txs, {} accounts touched)",
            height,
            summaries.len(),
            touched.len()
        );

        Ok(ProcessedBlock {
            block: BlockSummary {
                hash: block.hash,
                height,
                view: block.view as i64,
                parent: block.parent,
                proposer: block.proposer,
                timestamp_ms: block.timestamp_ms as i64,
                tx_count: summaries.len() as i32,
                justify_view: block.justify_view as i64,
            },
            transactions: summaries,
        })
    }

    /// Forget everything indexed and start over at height 0 for `chain_id`. Accounts, API keys
    /// and sessions are not chain data and survive.
    pub async fn reset_chain(&self, chain_id: i64) -> Result<()> {
        let mut conn = self.pool.inner().acquire().await?;
        db::reset_chain_data(&mut conn, chain_id).await?;
        warn!("chain data reset; re-indexing chain {} from height 0", chain_id);
        Ok(())
    }

    /// Fetch an account from the node and upsert it. Failures are logged, not fatal.
    pub async fn refresh_account(&self, address: &str, seen_height: i64) {
        match self.rpc.account(address).await {
            Ok(acc) => {
                let mut conn = match self.pool.inner().acquire().await {
                    Ok(c) => c,
                    Err(e) => {
                        warn!("db acquire failed: {}", e);
                        return;
                    }
                };
                if let Err(e) = db::upsert_account(
                    &mut conn,
                    address,
                    &acc.balance,
                    acc.nonce as i64,
                    seen_height,
                )
                .await
                {
                    warn!("upsert account {} failed: {}", address, e);
                }
                if let Err(e) = db::refresh_account_tx_count(&mut conn, address).await {
                    warn!("refresh tx_count {} failed: {}", address, e);
                }
            }
            Err(e) => warn!("account fetch {} failed: {}", address, e),
        }
    }

    /// Drop blocks at and above `height` so they are re-fetched.
    pub async fn rewind_to(&self, height: i64) -> Result<()> {
        let mut dbtx = self.pool.inner().begin().await?;
        let n = db::delete_blocks_from(&mut dbtx, height).await?;
        let last_hash = if height > 0 {
            db::get_block_hash_at(self.pool.inner(), height - 1).await?
        } else {
            None
        };
        db::set_next_height(&mut dbtx, height, last_hash.as_deref()).await?;
        dbtx.commit().await?;
        warn!("rewound {} block(s); next height {}", n, height);
        Ok(())
    }
}

/// The database columns one RPC transaction kind fills. Borrowed from the RPC value.
struct TxFields<'a> {
    kind: TxKind,
    /// What goes in the `kind` column: the enum name, or the node's own tag for unknown kinds.
    kind_tag: &'a str,
    /// Native (SHRUGG) recipient only; a bridge destination is a foreign address and goes in
    /// `bridge_to` so it never becomes an account row.
    to: Option<&'a str>,
    amount: Option<&'a str>,
    program: Option<&'a str>,
    base_pc: Option<i64>,
    words_len: Option<i64>,
    proof_len: Option<i64>,
    recipients: Vec<String>,
    asset: Option<&'a str>,
    bridge_amount: Option<&'a str>,
    to_chain: Option<i32>,
    bridge_to: Option<&'a str>,
    bridge_fee: Option<&'a str>,
    attestation: Option<&'a str>,
}

impl<'a> TxFields<'a> {
    fn empty(kind: TxKind) -> Self {
        TxFields {
            kind,
            kind_tag: kind.as_str(),
            to: None,
            amount: None,
            program: None,
            base_pc: None,
            words_len: None,
            proof_len: None,
            recipients: vec![],
            asset: None,
            bridge_amount: None,
            to_chain: None,
            bridge_to: None,
            bridge_fee: None,
            attestation: None,
        }
    }

    fn from_rpc(kind: &'a RpcTxKind) -> Self {
        match kind {
            RpcTxKind::Transfer { to, amount } => TxFields {
                to: Some(to),
                amount: Some(amount),
                ..Self::empty(TxKind::Transfer)
            },
            RpcTxKind::Mint { to, amount } => TxFields {
                to: Some(to),
                amount: Some(amount),
                ..Self::empty(TxKind::Mint)
            },
            RpcTxKind::Deploy { base_pc, words_len, program } => TxFields {
                program: Some(program),
                base_pc: Some(*base_pc as i64),
                words_len: Some(*words_len as i64),
                ..Self::empty(TxKind::Deploy)
            },
            RpcTxKind::Call { program, proof_len, recipients } => TxFields {
                program: Some(program),
                proof_len: Some(*proof_len as i64),
                recipients: recipients.clone(),
                ..Self::empty(TxKind::Call)
            },
            RpcTxKind::BridgeAttest { attestation } => TxFields {
                attestation: Some(attestation),
                ..Self::empty(TxKind::BridgeAttest)
            },
            RpcTxKind::BridgeBurn { asset, amount, to_chain, to, fee } => TxFields {
                asset: Some(asset),
                bridge_amount: Some(amount),
                to_chain: Some(i32::from(*to_chain)),
                bridge_to: Some(to),
                bridge_fee: Some(fee),
                ..Self::empty(TxKind::BridgeBurn)
            },
            RpcTxKind::Unknown { kind: tag } => TxFields {
                // The column is VARCHAR(32); a longer tag is stored truncated on a char boundary.
                kind_tag: truncate_chars(tag, 32),
                ..Self::empty(TxKind::Other)
            },
        }
    }
}

fn truncate_chars(s: &str, max: usize) -> &str {
    match s.char_indices().nth(max) {
        Some((i, _)) => &s[..i],
        None => s,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bridge_burn_keeps_foreign_address_out_of_to() {
        let k = RpcTxKind::BridgeBurn {
            asset: "8f1c".into(),
            amount: "500".into(),
            to_chain: 2,
            to: "00".repeat(32),
            fee: "5".into(),
        };
        let f = TxFields::from_rpc(&k);
        assert_eq!(f.kind, TxKind::BridgeBurn);
        assert_eq!(f.kind_tag, "bridge_burn");
        assert!(f.to.is_none() && f.amount.is_none());
        assert_eq!(f.bridge_amount, Some("500"));
        assert_eq!(f.to_chain, Some(2));
        assert_eq!(f.bridge_to.map(str::len), Some(64));
    }

    #[test]
    fn unknown_kind_is_stored_under_the_node_tag() {
        let k = RpcTxKind::Unknown { kind: "x".repeat(40) };
        let f = TxFields::from_rpc(&k);
        assert_eq!(f.kind, TxKind::Other);
        assert_eq!(f.kind_tag.len(), 32);
        let k = RpcTxKind::Unknown { kind: "shielded".into() };
        assert_eq!(TxFields::from_rpc(&k).kind_tag, "shielded");
    }
}
