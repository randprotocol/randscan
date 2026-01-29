//! Transaction queries

use crate::{CountRow, DbError, Result, TransactionRow};
use sqlx::PgPool;

/// Insert a transaction
pub async fn insert_transaction(
    pool: &PgPool,
    tx_id: &str,
    block_id: Option<&str>,
    block_height: Option<i64>,
    sender: &str,
    nonce: i64,
    compute_budget: i64,
    fee: i64,
    payload_type: &str,
    status: &str,
    timestamp: i64,
    signature: &str,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO transactions (
            tx_id, block_id, block_height, sender, nonce, compute_budget,
            fee, payload_type, status, timestamp, signature
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        ON CONFLICT (tx_id) DO UPDATE SET
            block_id = COALESCE(EXCLUDED.block_id, transactions.block_id),
            block_height = COALESCE(EXCLUDED.block_height, transactions.block_height),
            status = EXCLUDED.status
        "#,
    )
    .bind(tx_id)
    .bind(block_id)
    .bind(block_height)
    .bind(sender)
    .bind(nonce)
    .bind(compute_budget)
    .bind(fee)
    .bind(payload_type)
    .bind(status)
    .bind(timestamp)
    .bind(signature)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get transaction by ID
pub async fn get_transaction_by_id(pool: &PgPool, tx_id: &str) -> Result<TransactionRow> {
    sqlx::query_as::<_, TransactionRow>("SELECT * FROM transactions WHERE tx_id = $1")
        .bind(tx_id)
        .fetch_one(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                DbError::NotFound(format!("Transaction {} not found", tx_id))
            }
            _ => e.into(),
        })
}

/// Get transactions with pagination and filters
pub async fn get_transactions(
    pool: &PgPool,
    offset: i64,
    limit: i64,
    sender: Option<&str>,
    block_id: Option<&str>,
    payload_type: Option<&str>,
    status: Option<&str>,
    from_timestamp: Option<i64>,
    to_timestamp: Option<i64>,
) -> Result<Vec<TransactionRow>> {
    let mut query = String::from("SELECT * FROM transactions WHERE 1=1");
    let mut params: Vec<String> = Vec::new();

    if let Some(s) = sender {
        params.push(s.to_string());
        query.push_str(&format!(" AND sender = ${}", params.len()));
    }
    if let Some(b) = block_id {
        params.push(b.to_string());
        query.push_str(&format!(" AND block_id = ${}", params.len()));
    }
    if let Some(p) = payload_type {
        params.push(p.to_string());
        query.push_str(&format!(" AND payload_type = ${}", params.len()));
    }
    if let Some(st) = status {
        params.push(st.to_string());
        query.push_str(&format!(" AND status = ${}", params.len()));
    }

    query.push_str(" ORDER BY timestamp DESC");

    // We need to use a dynamic approach since sqlx doesn't support dynamic params well
    // For simplicity, we'll build the query based on what params are provided
    let rows = if sender.is_some()
        && block_id.is_none()
        && payload_type.is_none()
        && status.is_none()
    {
        sqlx::query_as::<_, TransactionRow>(
            "SELECT * FROM transactions WHERE sender = $1 ORDER BY timestamp DESC LIMIT $2 OFFSET $3",
        )
        .bind(sender.unwrap())
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?
    } else if block_id.is_some()
        && sender.is_none()
        && payload_type.is_none()
        && status.is_none()
    {
        sqlx::query_as::<_, TransactionRow>(
            "SELECT * FROM transactions WHERE block_id = $1 ORDER BY timestamp DESC LIMIT $2 OFFSET $3",
        )
        .bind(block_id.unwrap())
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?
    } else if payload_type.is_some()
        && sender.is_none()
        && block_id.is_none()
        && status.is_none()
    {
        sqlx::query_as::<_, TransactionRow>(
            "SELECT * FROM transactions WHERE payload_type = $1 ORDER BY timestamp DESC LIMIT $2 OFFSET $3",
        )
        .bind(payload_type.unwrap())
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?
    } else {
        // Default: no filters
        sqlx::query_as::<_, TransactionRow>(
            "SELECT * FROM transactions ORDER BY timestamp DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?
    };

    Ok(rows)
}

/// Count transactions with filters
pub async fn count_transactions(
    pool: &PgPool,
    sender: Option<&str>,
    block_id: Option<&str>,
    payload_type: Option<&str>,
    status: Option<&str>,
) -> Result<i64> {
    let row = if let Some(s) = sender {
        sqlx::query_as::<_, CountRow>(
            "SELECT COUNT(*) as count FROM transactions WHERE sender = $1",
        )
        .bind(s)
        .fetch_one(pool)
        .await?
    } else if let Some(b) = block_id {
        sqlx::query_as::<_, CountRow>(
            "SELECT COUNT(*) as count FROM transactions WHERE block_id = $1",
        )
        .bind(b)
        .fetch_one(pool)
        .await?
    } else if let Some(p) = payload_type {
        sqlx::query_as::<_, CountRow>(
            "SELECT COUNT(*) as count FROM transactions WHERE payload_type = $1",
        )
        .bind(p)
        .fetch_one(pool)
        .await?
    } else {
        sqlx::query_as::<_, CountRow>("SELECT COUNT(*) as count FROM transactions")
            .fetch_one(pool)
            .await?
    };

    Ok(row.value())
}

/// Get transactions for block
pub async fn get_transactions_for_block(
    pool: &PgPool,
    block_id: &str,
) -> Result<Vec<TransactionRow>> {
    sqlx::query_as::<_, TransactionRow>(
        "SELECT * FROM transactions WHERE block_id = $1 ORDER BY timestamp",
    )
    .bind(block_id)
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

/// Update transaction status
pub async fn update_transaction_status(pool: &PgPool, tx_id: &str, status: &str) -> Result<()> {
    sqlx::query("UPDATE transactions SET status = $1 WHERE tx_id = $2")
        .bind(status)
        .bind(tx_id)
        .execute(pool)
        .await?;

    Ok(())
}

/// Insert public transaction payload
pub async fn insert_tx_public(
    pool: &PgPool,
    tx_id: &str,
    instructions: &[u8],
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO tx_public (tx_id, instructions, instructions_size)
        VALUES ($1, $2, $3)
        ON CONFLICT (tx_id) DO NOTHING
        "#,
    )
    .bind(tx_id)
    .bind(instructions)
    .bind(instructions.len() as i32)
    .execute(pool)
    .await?;

    Ok(())
}

/// Insert private transaction payload
pub async fn insert_tx_private(
    pool: &PgPool,
    tx_id: &str,
    encrypted_payload: &[u8],
    proof: &[u8],
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO tx_private (tx_id, encrypted_payload, proof, encrypted_payload_size, proof_size)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (tx_id) DO NOTHING
        "#,
    )
    .bind(tx_id)
    .bind(encrypted_payload)
    .bind(proof)
    .bind(encrypted_payload.len() as i32)
    .bind(proof.len() as i32)
    .execute(pool)
    .await?;

    Ok(())
}

/// Insert stealth transaction payload
pub async fn insert_tx_stealth(
    pool: &PgPool,
    tx_id: &str,
    ephemeral_pubkey: &str,
    stealth_address: &str,
    encrypted_amount: &str,
    proof: &[u8],
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO tx_stealth (tx_id, ephemeral_pubkey, stealth_address, encrypted_amount, proof, proof_size)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (tx_id) DO NOTHING
        "#,
    )
    .bind(tx_id)
    .bind(ephemeral_pubkey)
    .bind(stealth_address)
    .bind(encrypted_amount)
    .bind(proof)
    .bind(proof.len() as i32)
    .execute(pool)
    .await?;

    Ok(())
}

/// Insert stake transaction payload
pub async fn insert_tx_stake(pool: &PgPool, tx_id: &str, amount: i64) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO tx_stake (tx_id, amount)
        VALUES ($1, $2)
        ON CONFLICT (tx_id) DO NOTHING
        "#,
    )
    .bind(tx_id)
    .bind(amount)
    .execute(pool)
    .await?;

    Ok(())
}

/// Insert unstake transaction payload
pub async fn insert_tx_unstake(pool: &PgPool, tx_id: &str, amount: i64) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO tx_unstake (tx_id, amount)
        VALUES ($1, $2)
        ON CONFLICT (tx_id) DO NOTHING
        "#,
    )
    .bind(tx_id)
    .bind(amount)
    .execute(pool)
    .await?;

    Ok(())
}

/// Insert transfer transaction payload
pub async fn insert_tx_transfer(
    pool: &PgPool,
    tx_id: &str,
    recipient: &str,
    amount: i64,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO tx_transfer (tx_id, recipient, amount)
        VALUES ($1, $2, $3)
        ON CONFLICT (tx_id) DO NOTHING
        "#,
    )
    .bind(tx_id)
    .bind(recipient)
    .bind(amount)
    .execute(pool)
    .await?;

    Ok(())
}

/// Insert deploy transaction payload
pub async fn insert_tx_deploy(
    pool: &PgPool,
    tx_id: &str,
    code: &[u8],
    program_id: Option<&str>,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO tx_deploy (tx_id, code, code_size, program_id)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (tx_id) DO NOTHING
        "#,
    )
    .bind(tx_id)
    .bind(code)
    .bind(code.len() as i32)
    .bind(program_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Insert invoke transaction payload
pub async fn insert_tx_invoke(
    pool: &PgPool,
    tx_id: &str,
    program_id: &str,
    instruction: &[u8],
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO tx_invoke (tx_id, program_id, instruction, instruction_size)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (tx_id) DO NOTHING
        "#,
    )
    .bind(tx_id)
    .bind(program_id)
    .bind(instruction)
    .bind(instruction.len() as i32)
    .execute(pool)
    .await?;

    Ok(())
}

/// Insert private transfer transaction payload
pub async fn insert_tx_private_transfer(pool: &PgPool, tx_id: &str, proof: &[u8]) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO tx_private_transfer (tx_id, proof, proof_size)
        VALUES ($1, $2, $3)
        ON CONFLICT (tx_id) DO NOTHING
        "#,
    )
    .bind(tx_id)
    .bind(proof)
    .bind(proof.len() as i32)
    .execute(pool)
    .await?;

    Ok(())
}

/// Search transactions by ID prefix
pub async fn search_transactions(pool: &PgPool, query: &str, limit: i64) -> Result<Vec<TransactionRow>> {
    let pattern = format!("{}%", query);
    sqlx::query_as::<_, TransactionRow>(
        "SELECT * FROM transactions WHERE tx_id LIKE $1 ORDER BY timestamp DESC LIMIT $2",
    )
    .bind(&pattern)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}
