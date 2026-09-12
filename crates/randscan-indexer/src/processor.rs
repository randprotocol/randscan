//! Turns an RPC block into database rows: the transaction (bundle + action), its nullifiers,
//! call receipts and deployed programs. There are no accounts to refresh on this chain.

use crate::rpc::{RpcAction, RpcBlock, RpcBundle, RpcClient};
use anyhow::{Context, Result};
use randscan_core::{BlockSummary, TransactionSummary, TxKind};
use randscan_db::{self as db, DbPool, NewBlock, NewBundle, NewTx};
use std::collections::HashMap;
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
    height: i64,
    index: i32,
    h_in: String,
}

/// What `shrugg_getProgram` adds to a deploy action: the code hash and the base pc.
struct ProgramMeta {
    base_pc: i64,
    code_hash: String,
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
            if let RpcAction::Call { .. } = tx.action {
                match self.rpc.receipt(&tx.hash).await {
                    Ok(Some(r)) => receipts.push(PreparedReceipt {
                        tx_hash: tx.hash.clone(),
                        program: r.program,
                        tier: r.tier as i32,
                        outputs: r.outputs,
                        height: r.height as i64,
                        index: r.index as i32,
                        h_in: r.h_in,
                    }),
                    Ok(None) => warn!("no receipt yet for call {} at height {}", tx.hash, height),
                    Err(e) => warn!("receipt fetch failed for {}: {}", tx.hash, e),
                }
            }
        }

        // A deploy action names the program id and its length; the code hash (a Poseidon2
        // digest, distinct from the content id) and the base pc come from the node's record.
        let mut programs: HashMap<String, ProgramMeta> = HashMap::new();
        for tx in &block.transactions {
            if let RpcAction::Deploy { program, .. } = &tx.action {
                let meta = match self.rpc.program(program).await {
                    Ok(Some(p)) => ProgramMeta {
                        base_pc: p.base_pc as i64,
                        code_hash: p.code_hash,
                    },
                    Ok(None) => ProgramMeta {
                        base_pc: 0,
                        code_hash: program.clone(),
                    },
                    Err(e) => {
                        warn!("program fetch {} failed: {}", program, e);
                        ProgramMeta {
                            base_pc: 0,
                            code_hash: program.clone(),
                        }
                    }
                };
                programs.insert(program.clone(), meta);
            }
        }

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
            let f = TxFields::from_action(&tx.action);
            let bundle = tx.bundle.as_ref().map(new_bundle);
            let fee = bundle
                .as_ref()
                .map(|b| b.fee.clone())
                .unwrap_or_else(|| "0".to_string());

            db::insert_transaction(
                &mut dbtx,
                &NewTx {
                    hash: &tx.hash,
                    block_hash: &block.hash,
                    height,
                    tx_index,
                    chain_id: tx.chain_id as i64,
                    timestamp_ms: block.timestamp_ms as i64,
                    kind: f.kind_tag,
                    bundle,
                    program_id: f.program,
                    words_len: f.words_len,
                    call_proof_len: f.call_proof_len,
                    input_envelope_len: f.input_envelope_len,
                    amount: f.amount.clone(),
                    cm: f.cm,
                    validator: f.validator,
                    registered: f.registered,
                    action_nonce: f.action_nonce,
                    attestation_len: f.attestation_len,
                    recipient: f.recipient,
                    note_time: f.note_time,
                    asset_index: f.asset_index,
                    relayer_fee: f.relayer_fee.clone(),
                    to_chain: f.to_chain,
                    bridge_to: f.bridge_to,
                    asset_bundle: f.asset_bundle.map(new_bundle),
                },
            )
            .await
            .with_context(|| format!("insert tx {}", tx.hash))?;

            if let RpcAction::Deploy { program, words } = &tx.action {
                let meta = programs.get(program);
                db::insert_program(
                    &mut dbtx,
                    program,
                    &tx.hash,
                    height,
                    meta.map(|m| m.base_pc).unwrap_or(0),
                    *words as i64,
                    meta.map(|m| m.code_hash.as_str()).unwrap_or(program),
                )
                .await?;
            }

            summaries.push(TransactionSummary {
                hash: tx.hash.clone(),
                height,
                block_hash: block.hash.clone(),
                tx_index,
                kind: f.kind,
                fee,
                timestamp_ms: block.timestamp_ms as i64,
                has_bundle: tx.bundle.is_some(),
                program: f.program.map(str::to_string),
                validator: f.validator.map(str::to_string),
                amount: f.amount,
                asset_index: f.asset_index,
            });
        }

        for r in &receipts {
            db::insert_receipt(
                &mut dbtx, &r.tx_hash, &r.program, r.tier, &r.outputs, r.height, r.index, &r.h_in,
            )
            .await?;
        }

        // Leaves of this height fetched before the block (after a rewind) get their tx now.
        db::link_notes_at(&mut dbtx, height).await?;
        db::set_next_height(&mut dbtx, height + 1, Some(&block.hash)).await?;
        dbtx.commit().await?;

        debug!("indexed block {} ({} txs)", height, summaries.len());

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

    /// Fetch the commitment tree from leaf `next_leaf` onwards, a page at a time, until the
    /// node's reply is short. Returns the next leaf to fetch.
    pub async fn sync_notes(&self, mut next_leaf: i64) -> Result<i64> {
        const PAGE: u64 = 1000;
        loop {
            let rows = self.rpc.commitments(next_leaf as u64, PAGE).await?;
            if rows.is_empty() {
                return Ok(next_leaf);
            }
            let batch: Vec<db::NewNote> = rows
                .iter()
                .map(|r| db::NewNote {
                    leaf_index: r.index as i64,
                    cm: r.cm.clone(),
                    height: r.height as i64,
                    envelope: r.envelope.clone(),
                })
                .collect();
            let last = batch.last().map(|r| r.leaf_index).unwrap_or(next_leaf);
            let mut dbtx = self.pool.inner().begin().await?;
            db::insert_notes(&mut dbtx, &batch).await?;
            db::set_next_leaf(&mut dbtx, last + 1).await?;
            dbtx.commit().await?;
            debug!("indexed notes {}..={}", next_leaf, last);
            next_leaf = last + 1;
            if (rows.len() as u64) < PAGE {
                return Ok(next_leaf);
            }
        }
    }

    /// Forget everything indexed and start over at height 0 for `chain_id`. Users, API keys
    /// and sessions are not chain data and survive.
    pub async fn reset_chain(&self, chain_id: i64) -> Result<()> {
        let mut conn = self.pool.inner().acquire().await?;
        db::reset_chain_data(&mut conn, chain_id).await?;
        warn!(
            "chain data reset; re-indexing chain {} from height 0",
            chain_id
        );
        Ok(())
    }

    /// Drop blocks at and above `height` so they are re-fetched. Tree leaves of those heights go
    /// too: the tree past a lost block is not the tree the node will serve.
    pub async fn rewind_to(&self, height: i64) -> Result<()> {
        let mut dbtx = self.pool.inner().begin().await?;
        let n = db::delete_blocks_from(&mut dbtx, height).await?;
        let leaves = db::delete_notes_from(&mut dbtx, height).await?;
        let next_leaf = db::next_leaf_after_rewind(&mut dbtx).await?;
        db::set_next_leaf(&mut dbtx, next_leaf).await?;
        let last_hash = if height > 0 {
            db::get_block_hash_at(self.pool.inner(), height - 1).await?
        } else {
            None
        };
        db::set_next_height(&mut dbtx, height, last_hash.as_deref()).await?;
        dbtx.commit().await?;
        warn!(
            "rewound {} block(s) and {} leaves; next height {}, next leaf {}",
            n, leaves, height, next_leaf
        );
        Ok(())
    }
}

fn new_bundle(b: &RpcBundle) -> NewBundle {
    NewBundle {
        anchor: b.anchor.clone(),
        nullifiers: b.nullifiers.clone(),
        commitments: b.commitments.clone(),
        fee: b.fee.0.clone(),
        burn: b.burn.0.clone(),
        asset: b.asset as i64,
        time: b.time as i64,
        proof_len: b.proof_len as i64,
        envelope_len: [b.envelope_len[0] as i64, b.envelope_len[1] as i64],
    }
}

/// The action columns one RPC action fills. Borrowed from the RPC value.
struct TxFields<'a> {
    kind: TxKind,
    /// What goes in the `kind` column: the explorer kind, or the node's own tag for unknown kinds.
    kind_tag: &'a str,
    program: Option<&'a str>,
    words_len: Option<i64>,
    call_proof_len: Option<i64>,
    input_envelope_len: Option<i64>,
    amount: Option<String>,
    cm: Option<&'a str>,
    validator: Option<&'a str>,
    registered: Option<bool>,
    action_nonce: Option<i64>,
    attestation_len: Option<i64>,
    recipient: Option<&'a str>,
    note_time: Option<i64>,
    asset_index: Option<i64>,
    relayer_fee: Option<String>,
    to_chain: Option<i32>,
    bridge_to: Option<&'a str>,
    asset_bundle: Option<&'a RpcBundle>,
}

impl<'a> TxFields<'a> {
    fn empty(kind: TxKind) -> Self {
        TxFields {
            kind,
            kind_tag: kind.as_str(),
            program: None,
            words_len: None,
            call_proof_len: None,
            input_envelope_len: None,
            amount: None,
            cm: None,
            validator: None,
            registered: None,
            action_nonce: None,
            attestation_len: None,
            recipient: None,
            note_time: None,
            asset_index: None,
            relayer_fee: None,
            to_chain: None,
            bridge_to: None,
            asset_bundle: None,
        }
    }

    fn from_action(action: &'a RpcAction) -> Self {
        match action {
            RpcAction::None => Self::empty(TxKind::Transfer),
            RpcAction::Mint { cm, amount, minter } => TxFields {
                cm: Some(cm),
                amount: Some(amount.0.clone()),
                validator: Some(minter),
                ..Self::empty(TxKind::Mint)
            },
            RpcAction::Deploy { program, words } => TxFields {
                program: Some(program),
                words_len: Some(*words as i64),
                ..Self::empty(TxKind::Deploy)
            },
            RpcAction::Call {
                program,
                proof_len,
                input_envelope_len,
            } => TxFields {
                program: Some(program),
                call_proof_len: Some(*proof_len as i64),
                input_envelope_len: input_envelope_len.map(|n| n as i64),
                ..Self::empty(TxKind::Call)
            },
            RpcAction::Bond {
                validator,
                amount,
                registered,
            } => TxFields {
                validator: Some(validator),
                amount: Some(amount.0.clone()),
                registered: Some(*registered),
                ..Self::empty(TxKind::Bond)
            },
            RpcAction::Unbond {
                validator,
                amount,
                nonce,
            } => TxFields {
                validator: Some(validator),
                amount: Some(amount.0.clone()),
                action_nonce: Some(*nonce as i64),
                ..Self::empty(TxKind::Unbond)
            },
            RpcAction::Withdraw {
                validator,
                amount,
                nonce,
            } => TxFields {
                validator: Some(validator),
                amount: Some(amount.0.clone()),
                action_nonce: Some(*nonce as i64),
                ..Self::empty(TxKind::Withdraw)
            },
            RpcAction::BridgeAttest {
                attestation_len,
                recipient,
                asset_index,
                amount,
                time,
            } => TxFields {
                attestation_len: Some(*attestation_len as i64),
                recipient: Some(recipient),
                asset_index: asset_index.map(|a| a as i64),
                amount: amount.as_ref().map(|a| a.0.clone()),
                note_time: time.map(|t| t as i64),
                ..Self::empty(TxKind::BridgeAttest)
            },
            RpcAction::BridgeBurn {
                asset,
                amount,
                relayer_fee,
                to_chain,
                to,
                asset_bundle,
            } => TxFields {
                asset_index: Some(*asset as i64),
                amount: Some(amount.0.clone()),
                relayer_fee: Some(relayer_fee.0.clone()),
                to_chain: Some(i32::from(*to_chain)),
                bridge_to: Some(to),
                asset_bundle: Some(asset_bundle),
                ..Self::empty(TxKind::BridgeBurn)
            },
            RpcAction::Unknown { kind: tag } => TxFields {
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
    use crate::rpc::Units;

    #[test]
    fn a_plain_transfer_is_kind_transfer_with_no_action_fields() {
        let f = TxFields::from_action(&RpcAction::None);
        assert_eq!(f.kind, TxKind::Transfer);
        assert_eq!(f.kind_tag, "transfer");
        assert!(f.amount.is_none() && f.validator.is_none() && f.program.is_none());
    }

    #[test]
    fn a_burn_keeps_the_foreign_address_and_asset_index() {
        let asset_bundle = RpcBundle {
            anchor: "a".into(),
            nullifiers: ["n1".into(), "n2".into()],
            commitments: ["c1".into(), "c2".into()],
            fee: Units("0".into()),
            burn: Units("500".into()),
            asset: 2,
            time: 9,
            proof_len: 1,
            envelope_len: [1, 1],
        };
        let k = RpcAction::BridgeBurn {
            asset: 2,
            amount: Units("400".into()),
            relayer_fee: Units("100".into()),
            to_chain: 5,
            to: "00".repeat(32),
            asset_bundle: Box::new(asset_bundle),
        };
        let f = TxFields::from_action(&k);
        assert_eq!(f.kind, TxKind::BridgeBurn);
        assert_eq!(f.asset_index, Some(2));
        assert_eq!(f.amount.as_deref(), Some("400"));
        assert_eq!(f.relayer_fee.as_deref(), Some("100"));
        assert_eq!(f.to_chain, Some(5));
        assert_eq!(f.bridge_to.map(str::len), Some(64));
        assert_eq!(f.asset_bundle.unwrap().burn.0, "500");
    }

    #[test]
    fn a_mint_records_the_minting_validator_as_validator() {
        let k = RpcAction::Mint {
            cm: "cm".into(),
            amount: Units("7".into()),
            minter: "2nRd".into(),
        };
        let f = TxFields::from_action(&k);
        assert_eq!(f.kind, TxKind::Mint);
        assert_eq!(f.validator, Some("2nRd"));
        assert_eq!(f.cm, Some("cm"));
    }

    #[test]
    fn unknown_kind_is_stored_under_the_node_tag() {
        let k = RpcAction::Unknown {
            kind: "x".repeat(40),
        };
        let f = TxFields::from_action(&k);
        assert_eq!(f.kind, TxKind::Other);
        assert_eq!(f.kind_tag.len(), 32);
        let k = RpcAction::Unknown {
            kind: "slash".into(),
        };
        assert_eq!(TxFields::from_action(&k).kind_tag, "slash");
    }
}
