//! Block and transaction processor

use crate::rpc::{BlockResponse, TransactionPayloadResponse, TransactionResponse};
use crate::broadcast::Broadcaster;
use randscan_core::{BlockSummary, TransactionSummary, PayloadType, Account, Validator};
use randscan_db::{self as db, DbPool};
use anyhow::Result;
use tracing::{info, warn, debug};

/// Block processor
pub struct BlockProcessor {
    pool: DbPool,
    broadcaster: Option<Broadcaster>,
}

impl BlockProcessor {
    pub fn new(pool: DbPool, broadcaster: Option<Broadcaster>) -> Self {
        Self { pool, broadcaster }
    }

    /// Process a block from RPC response
    pub async fn process_block(&self, block: BlockResponse) -> Result<()> {
        let db = self.pool.inner();

        // Insert block
        db::insert_block(
            db,
            &block.block_id,
            block.height as i64,
            block.view as i64,
            block.epoch as i64,
            &block.parent_id,
            &block.proposer,
            &block.transactions_root,
            &block.state_root,
            block.supply_commitment.as_deref().unwrap_or(""),
            block.timestamp as i64,
            block.transaction_count as i32,
            block.finalized,
        )
        .await?;

        // Insert QC if present
        if let (Some(vote_type), Some(view), Some(qc_block_id)) =
            (&block.qc_vote_type, block.qc_view, &block.qc_block_id)
        {
            db::insert_qc(
                db,
                &block.block_id,
                vote_type,
                view as i64,
                qc_block_id,
                block.height.saturating_sub(1) as i64,
                block.qc_signers.as_ref().map(|s| s.len()).unwrap_or(0) as i32,
            )
            .await?;

            // Insert QC signers
            if let Some(signers) = &block.qc_signers {
                for signer in signers {
                    db::insert_qc_signer(db, &block.block_id, signer, "").await?;
                }
            }
        }

        // Update validator blocks produced
        db::increment_blocks_produced(db, &block.proposer).await.ok();

        // Process transactions
        for tx in &block.transactions {
            self.process_transaction(tx, Some(&block.block_id), Some(block.height))
                .await?;
        }

        // Update epoch counters
        db::increment_epoch_counters(
            db,
            block.epoch as i64,
            1,
            block.transaction_count as i32,
        )
        .await
        .ok();

        // Broadcast new block
        if let Some(ref broadcaster) = self.broadcaster {
            let summary = BlockSummary {
                block_id: block.block_id.clone(),
                height: block.height,
                view: block.view,
                epoch: block.epoch,
                parent_id: block.parent_id.clone(),
                proposer: block.proposer.clone(),
                timestamp: block.timestamp,
                transaction_count: block.transaction_count as i32,
                finalized: block.finalized,
            };
            broadcaster.broadcast_block(summary);
        }

        debug!("Processed block {} at height {}", block.block_id, block.height);
        Ok(())
    }

    /// Process a transaction
    pub async fn process_transaction(
        &self,
        tx: &TransactionResponse,
        block_id: Option<&str>,
        block_height: Option<u64>,
    ) -> Result<()> {
        let db = self.pool.inner();
        let tx_type = &tx.tx_type;
        let status = &tx.status;

        // Insert main transaction record
        db::insert_transaction(
            db,
            &tx.signature,
            block_id,
            block_height.map(|h| h as i64),
            &tx.sender,
            tx.nonce as i64,
            tx.compute_budget.unwrap_or(0) as i64,
            tx.fee as i64,
            tx_type,
            status,
            tx.timestamp as i64,
            "", // signature stored separately
        )
        .await?;

        // Process type-specific payload
        if let Some(payload) = &tx.payload {
            self.process_payload(&tx.signature, payload).await?;
        }

        // Ensure sender account exists
        db::upsert_account(
            db,
            &tx.sender,
            0,
            0,
            tx.nonce as i64,
            false,
            None,
            0,
            tx.timestamp as i64,
        )
        .await?;

        // Link transaction to sender account
        if let Some(height) = block_height {
            db::insert_account_transaction(
                db,
                &tx.sender,
                &tx.signature,
                "sender",
                height as i64,
                tx.timestamp as i64,
            )
            .await?;
        }

        // Increment sender tx count
        db::increment_account_tx_count(db, &tx.sender).await.ok();

        // Broadcast new transaction
        if let Some(ref broadcaster) = self.broadcaster {
            let payload_type = PayloadType::from_str(tx_type)
                .unwrap_or(PayloadType::Public);

            let summary = TransactionSummary {
                tx_id: tx.signature.clone(),
                block_id: block_id.map(|s| s.to_string()),
                block_height: block_height.map(|h| h as i64),
                sender: tx.sender.clone(),
                nonce: tx.nonce as i64,
                fee: tx.fee as i64,
                payload_type: tx_type.clone(),
                status: status.clone(),
                timestamp: tx.timestamp as i64,
                privacy_level: payload_type.privacy_level().as_str().to_string(),
            };
            broadcaster.broadcast_transaction(summary);
        }

        Ok(())
    }

    /// Process transaction payload details
    async fn process_payload(
        &self,
        tx_id: &str,
        payload: &TransactionPayloadResponse,
    ) -> Result<()> {
        let db = self.pool.inner();

        match payload {
            TransactionPayloadResponse::Public { .. } => {
                db::insert_tx_public(db, tx_id, &[]).await?;
            }
            TransactionPayloadResponse::Private { nullifiers, commitments, .. } => {
                db::insert_tx_private(db, tx_id, &[], &[]).await?;

                // Track nullifiers and commitments
                for n in nullifiers {
                    db::insert_nullifier(db, n, tx_id).await?;
                }
                for c in commitments {
                    db::insert_commitment(db, c, tx_id).await?;
                }
            }
            TransactionPayloadResponse::Stealth { ephemeral_pubkey, stealth_address, .. } => {
                db::insert_tx_stealth(
                    db,
                    tx_id,
                    ephemeral_pubkey,
                    stealth_address,
                    "",
                    &[],
                )
                .await?;
            }
            TransactionPayloadResponse::Stake { amount } => {
                db::insert_tx_stake(db, tx_id, *amount as i64).await?;
            }
            TransactionPayloadResponse::Unstake { amount } => {
                db::insert_tx_unstake(db, tx_id, *amount as i64).await?;
            }
            TransactionPayloadResponse::Transfer { to, amount } => {
                db::insert_tx_transfer(db, tx_id, to, *amount as i64).await?;

                // Ensure recipient account exists
                db::upsert_account(db, to, 0, 0, 0, false, None, 0, 0).await.ok();
            }
            TransactionPayloadResponse::Deploy { program_id, .. } => {
                db::insert_tx_deploy(db, tx_id, &[], program_id.as_deref()).await?;
            }
            TransactionPayloadResponse::Invoke { program_id, .. } => {
                db::insert_tx_invoke(db, tx_id, program_id, &[]).await?;
            }
            TransactionPayloadResponse::PrivateTransfer { nullifiers, commitments, .. } => {
                db::insert_tx_private_transfer(db, tx_id, &[]).await?;

                for n in nullifiers {
                    db::insert_nullifier(db, n, tx_id).await?;
                }
                for c in commitments {
                    db::insert_commitment(db, c, tx_id).await?;
                }
            }
            TransactionPayloadResponse::Mint { to, amount } => {
                // Mint is similar to transfer - it credits tokens to an address
                db::insert_tx_transfer(db, tx_id, to, *amount as i64).await?;

                // Ensure recipient account exists
                db::upsert_account(db, to, 0, 0, 0, false, None, 0, 0).await.ok();
            }
        }

        Ok(())
    }

    /// Update account from RPC response
    pub async fn update_account(&self, address: &str, info: &crate::rpc::AccountInfoResponse) -> Result<()> {
        let db = self.pool.inner();

        db::upsert_account(
            db,
            address,
            info.atlas_balance as i64,
            info.shrug_balance as i64,
            info.nonce as i64,
            info.executable,
            info.owner.as_deref(),
            info.data_len as i32,
            chrono::Utc::now().timestamp_millis(),
        )
        .await?;

        // Broadcast account update
        if let Some(ref broadcaster) = self.broadcaster {
            let account = Account {
                address: address.to_string(),
                atlas_balance: info.atlas_balance as i64,
                shrug_balance: info.shrug_balance as i64,
                nonce: info.nonce as i64,
                is_executable: info.executable,
                owner: info.owner.clone(),
                data_len: info.data_len as i32,
                tx_count: 0,
                first_seen: 0,
                last_seen: chrono::Utc::now().timestamp_millis(),
            };
            broadcaster.broadcast_account_update(account);
        }

        Ok(())
    }

    /// Update validator from RPC response
    pub async fn update_validator(&self, val: &crate::rpc::ValidatorResponse) -> Result<()> {
        let db = self.pool.inner();
        let now = chrono::Utc::now().timestamp_millis();

        db::upsert_validator(
            db,
            &val.validator_id,
            &val.pubkey,
            val.stake as i64,
            val.commission_rate as i16,
            val.is_active,
            now,
        )
        .await?;

        if let Some(vote_height) = val.last_vote_height {
            db::update_last_vote_height(db, &val.validator_id, vote_height as i64).await?;
        }

        // Broadcast validator update
        if let Some(ref broadcaster) = self.broadcaster {
            let validator = Validator {
                validator_id: val.validator_id.clone(),
                pubkey: val.pubkey.clone(),
                stake: val.stake as i64,
                commission_rate: val.commission_rate as i16,
                is_active: val.is_active,
                blocks_produced: val.blocks_produced.unwrap_or(0) as i64,
                blocks_skipped: 0,
                last_vote_height: val.last_vote_height.map(|h| h as i64),
                uptime_percentage: 100.0,
                first_seen: now,
                last_seen: now,
            };
            broadcaster.broadcast_validator_update(validator);
        }

        Ok(())
    }

    /// Finalize blocks up to given height
    pub async fn finalize_blocks(&self, height: u64) -> Result<i64> {
        let db = self.pool.inner();
        let count = db::finalize_blocks_up_to(db, height as i64).await?;

        if count > 0 {
            info!("Finalized {} blocks up to height {}", count, height);
        }

        Ok(count)
    }
}
