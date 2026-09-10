//! Turns an RPC block into database rows and refreshes the touched accounts.

use crate::rpc::{RpcBlock, RpcClient, RpcTxKind};
use anyhow::{Context, Result};
use randscan_core::{BlockSummary, TransactionSummary, TxKind};
use randscan_db::{self as db, DbPool, NewBlock, NewTx};
use std::collections::BTreeSet;
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
            #[allow(clippy::type_complexity)]
            let (kind, to, amount, program, base_pc, words_len, proof_len, recipients): (
                TxKind,
                Option<&str>,
                Option<&str>,
                Option<&str>,
                Option<i64>,
                Option<i64>,
                Option<i64>,
                Vec<String>,
            ) = match &tx.kind {
                RpcTxKind::Transfer { to, amount } => {
                    (TxKind::Transfer, Some(to), Some(amount), None, None, None, None, vec![])
                }
                RpcTxKind::Mint { to, amount } => {
                    (TxKind::Mint, Some(to), Some(amount), None, None, None, None, vec![])
                }
                RpcTxKind::Deploy { base_pc, words_len, program } => (
                    TxKind::Deploy,
                    None,
                    None,
                    Some(program),
                    Some(*base_pc as i64),
                    Some(*words_len as i64),
                    None,
                    vec![],
                ),
                RpcTxKind::Call { program, proof_len, recipients } => (
                    TxKind::Call,
                    None,
                    None,
                    Some(program),
                    None,
                    None,
                    Some(*proof_len as i64),
                    recipients.clone(),
                ),
            };

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
                    kind: kind.as_str(),
                    chain_id: tx.chain_id as i64,
                    timestamp_ms: block.timestamp_ms as i64,
                    to_address: to,
                    amount,
                    program_id: program,
                    base_pc,
                    words_len,
                    proof_len,
                    recipients: &recipients,
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
                    program,
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
