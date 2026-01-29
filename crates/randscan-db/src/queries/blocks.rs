//! Block queries

use crate::{BlockRow, CountRow, DbError, QcRow, QcSignerRow, Result};
use sqlx::PgPool;

/// Insert a new block
pub async fn insert_block(
    pool: &PgPool,
    block_id: &str,
    height: i64,
    view_number: i64,
    epoch: i64,
    parent_id: &str,
    proposer_id: &str,
    transactions_root: &str,
    state_root: &str,
    supply_commitment: &str,
    timestamp: i64,
    transaction_count: i32,
    finalized: bool,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO blocks (
            block_id, height, view_number, epoch, parent_id, proposer_id,
            transactions_root, state_root, supply_commitment, timestamp,
            transaction_count, finalized
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        ON CONFLICT (block_id) DO UPDATE SET
            finalized = EXCLUDED.finalized,
            transaction_count = EXCLUDED.transaction_count
        "#,
    )
    .bind(block_id)
    .bind(height)
    .bind(view_number)
    .bind(epoch)
    .bind(parent_id)
    .bind(proposer_id)
    .bind(transactions_root)
    .bind(state_root)
    .bind(supply_commitment)
    .bind(timestamp)
    .bind(transaction_count)
    .bind(finalized)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get block by ID
pub async fn get_block_by_id(pool: &PgPool, block_id: &str) -> Result<BlockRow> {
    sqlx::query_as::<_, BlockRow>(
        "SELECT * FROM blocks WHERE block_id = $1",
    )
    .bind(block_id)
    .fetch_one(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => DbError::NotFound(format!("Block {} not found", block_id)),
        _ => e.into(),
    })
}

/// Get block by height
pub async fn get_block_by_height(pool: &PgPool, height: i64) -> Result<BlockRow> {
    sqlx::query_as::<_, BlockRow>(
        "SELECT * FROM blocks WHERE height = $1",
    )
    .bind(height)
    .fetch_one(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => DbError::NotFound(format!("Block at height {} not found", height)),
        _ => e.into(),
    })
}

/// Get latest block
pub async fn get_latest_block(pool: &PgPool) -> Result<BlockRow> {
    sqlx::query_as::<_, BlockRow>(
        "SELECT * FROM blocks ORDER BY height DESC LIMIT 1",
    )
    .fetch_one(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => DbError::NotFound("No blocks found".to_string()),
        _ => e.into(),
    })
}

/// Get blocks with pagination
pub async fn get_blocks(
    pool: &PgPool,
    offset: i64,
    limit: i64,
    proposer: Option<&str>,
    epoch: Option<i64>,
    finalized: Option<bool>,
) -> Result<Vec<BlockRow>> {
    let mut query = String::from("SELECT * FROM blocks WHERE 1=1");
    let mut param_count = 0;

    if proposer.is_some() {
        param_count += 1;
        query.push_str(&format!(" AND proposer_id = ${}", param_count));
    }
    if epoch.is_some() {
        param_count += 1;
        query.push_str(&format!(" AND epoch = ${}", param_count));
    }
    if finalized.is_some() {
        param_count += 1;
        query.push_str(&format!(" AND finalized = ${}", param_count));
    }

    query.push_str(&format!(" ORDER BY height DESC LIMIT ${} OFFSET ${}", param_count + 1, param_count + 2));

    let mut q = sqlx::query_as::<_, BlockRow>(&query);

    if let Some(p) = proposer {
        q = q.bind(p);
    }
    if let Some(e) = epoch {
        q = q.bind(e);
    }
    if let Some(f) = finalized {
        q = q.bind(f);
    }

    q = q.bind(limit).bind(offset);

    q.fetch_all(pool).await.map_err(Into::into)
}

/// Count blocks
pub async fn count_blocks(
    pool: &PgPool,
    proposer: Option<&str>,
    epoch: Option<i64>,
    finalized: Option<bool>,
) -> Result<i64> {
    let mut query = String::from("SELECT COUNT(*) as count FROM blocks WHERE 1=1");
    let mut param_count = 0;

    if proposer.is_some() {
        param_count += 1;
        query.push_str(&format!(" AND proposer_id = ${}", param_count));
    }
    if epoch.is_some() {
        param_count += 1;
        query.push_str(&format!(" AND epoch = ${}", param_count));
    }
    if finalized.is_some() {
        param_count += 1;
        query.push_str(&format!(" AND finalized = ${}", param_count));
    }

    let mut q = sqlx::query_as::<_, CountRow>(&query);

    if let Some(p) = proposer {
        q = q.bind(p);
    }
    if let Some(e) = epoch {
        q = q.bind(e);
    }
    if let Some(f) = finalized {
        q = q.bind(f);
    }

    let row = q.fetch_one(pool).await?;
    Ok(row.value())
}

/// Update block finalized status
pub async fn update_block_finalized(pool: &PgPool, block_id: &str, finalized: bool) -> Result<()> {
    sqlx::query("UPDATE blocks SET finalized = $1 WHERE block_id = $2")
        .bind(finalized)
        .bind(block_id)
        .execute(pool)
        .await?;

    Ok(())
}

/// Finalize blocks up to height
pub async fn finalize_blocks_up_to(pool: &PgPool, height: i64) -> Result<i64> {
    let result = sqlx::query(
        "UPDATE blocks SET finalized = TRUE WHERE height <= $1 AND finalized = FALSE",
    )
    .bind(height)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() as i64)
}

/// Insert quorum certificate
pub async fn insert_qc(
    pool: &PgPool,
    block_id: &str,
    vote_type: &str,
    view_number: i64,
    certified_block_id: &str,
    certified_block_height: i64,
    signer_count: i32,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO quorum_certificates (
            block_id, vote_type, view_number, certified_block_id,
            certified_block_height, signer_count
        ) VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (block_id) DO UPDATE SET
            signer_count = EXCLUDED.signer_count
        "#,
    )
    .bind(block_id)
    .bind(vote_type)
    .bind(view_number)
    .bind(certified_block_id)
    .bind(certified_block_height)
    .bind(signer_count)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get QC for block
pub async fn get_qc_for_block(pool: &PgPool, block_id: &str) -> Result<QcRow> {
    sqlx::query_as::<_, QcRow>(
        "SELECT * FROM quorum_certificates WHERE block_id = $1",
    )
    .bind(block_id)
    .fetch_one(pool)
    .await
    .map_err(Into::into)
}

/// Insert QC signer
pub async fn insert_qc_signer(
    pool: &PgPool,
    block_id: &str,
    validator_id: &str,
    signature: &str,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO qc_signers (block_id, validator_id, signature)
        VALUES ($1, $2, $3)
        ON CONFLICT (block_id, validator_id) DO NOTHING
        "#,
    )
    .bind(block_id)
    .bind(validator_id)
    .bind(signature)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get QC signers for block
pub async fn get_qc_signers(pool: &PgPool, block_id: &str) -> Result<Vec<QcSignerRow>> {
    sqlx::query_as::<_, QcSignerRow>(
        "SELECT * FROM qc_signers WHERE block_id = $1",
    )
    .bind(block_id)
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

/// Get blocks by proposer
pub async fn get_blocks_by_proposer(
    pool: &PgPool,
    proposer_id: &str,
    limit: i64,
) -> Result<Vec<BlockRow>> {
    sqlx::query_as::<_, BlockRow>(
        "SELECT * FROM blocks WHERE proposer_id = $1 ORDER BY height DESC LIMIT $2",
    )
    .bind(proposer_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

/// Check if block exists
pub async fn block_exists(pool: &PgPool, block_id: &str) -> Result<bool> {
    let row = sqlx::query_as::<_, CountRow>(
        "SELECT COUNT(*) as count FROM blocks WHERE block_id = $1",
    )
    .bind(block_id)
    .fetch_one(pool)
    .await?;

    Ok(row.value() > 0)
}

/// Delete blocks above height (for reorg handling)
pub async fn delete_blocks_above_height(pool: &PgPool, height: i64) -> Result<i64> {
    let result = sqlx::query("DELETE FROM blocks WHERE height > $1")
        .bind(height)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() as i64)
}
