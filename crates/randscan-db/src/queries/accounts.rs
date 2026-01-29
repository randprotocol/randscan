//! Account queries

use crate::{AccountRow, AccountTransactionRow, CountRow, DbError, Result};
use sqlx::PgPool;

/// Insert or update an account
pub async fn upsert_account(
    pool: &PgPool,
    address: &str,
    atlas_balance: i64,
    shrug_balance: i64,
    nonce: i64,
    is_executable: bool,
    owner: Option<&str>,
    data_len: i32,
    timestamp: i64,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO accounts (
            address, atlas_balance, shrug_balance, nonce, is_executable,
            owner, data_len, tx_count, first_seen, last_seen
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, 0, $8, $8)
        ON CONFLICT (address) DO UPDATE SET
            atlas_balance = EXCLUDED.atlas_balance,
            shrug_balance = EXCLUDED.shrug_balance,
            nonce = GREATEST(accounts.nonce, EXCLUDED.nonce),
            is_executable = EXCLUDED.is_executable,
            owner = COALESCE(EXCLUDED.owner, accounts.owner),
            data_len = EXCLUDED.data_len,
            last_seen = EXCLUDED.last_seen
        "#,
    )
    .bind(address)
    .bind(atlas_balance)
    .bind(shrug_balance)
    .bind(nonce)
    .bind(is_executable)
    .bind(owner)
    .bind(data_len)
    .bind(timestamp)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get account by address
pub async fn get_account_by_address(pool: &PgPool, address: &str) -> Result<AccountRow> {
    sqlx::query_as::<_, AccountRow>("SELECT * FROM accounts WHERE address = $1")
        .bind(address)
        .fetch_one(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                DbError::NotFound(format!("Account {} not found", address))
            }
            _ => e.into(),
        })
}

/// Get accounts with pagination
pub async fn get_accounts(
    pool: &PgPool,
    offset: i64,
    limit: i64,
    sort_by: Option<&str>,
) -> Result<Vec<AccountRow>> {
    let order_clause = match sort_by {
        Some("atlas_balance") => "ORDER BY atlas_balance DESC",
        Some("shrug_balance") => "ORDER BY shrug_balance DESC",
        Some("tx_count") => "ORDER BY tx_count DESC",
        Some("last_seen") => "ORDER BY last_seen DESC",
        _ => "ORDER BY last_seen DESC",
    };

    let query = format!(
        "SELECT * FROM accounts {} LIMIT $1 OFFSET $2",
        order_clause
    );

    sqlx::query_as::<_, AccountRow>(&query)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(Into::into)
}

/// Count accounts
pub async fn count_accounts(pool: &PgPool) -> Result<i64> {
    let row = sqlx::query_as::<_, CountRow>("SELECT COUNT(*) as count FROM accounts")
        .fetch_one(pool)
        .await?;

    Ok(row.value())
}

/// Update account balance
pub async fn update_account_balance(
    pool: &PgPool,
    address: &str,
    atlas_delta: i64,
    shrug_delta: i64,
) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE accounts SET
            atlas_balance = atlas_balance + $2,
            shrug_balance = shrug_balance + $3
        WHERE address = $1
        "#,
    )
    .bind(address)
    .bind(atlas_delta)
    .bind(shrug_delta)
    .execute(pool)
    .await?;

    Ok(())
}

/// Increment account transaction count
pub async fn increment_account_tx_count(pool: &PgPool, address: &str) -> Result<()> {
    sqlx::query("UPDATE accounts SET tx_count = tx_count + 1 WHERE address = $1")
        .bind(address)
        .execute(pool)
        .await?;

    Ok(())
}

/// Insert account transaction relationship
pub async fn insert_account_transaction(
    pool: &PgPool,
    account: &str,
    tx_id: &str,
    role: &str,
    block_height: i64,
    timestamp: i64,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO account_transactions (account, tx_id, role, block_height, timestamp)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (account, tx_id, role) DO NOTHING
        "#,
    )
    .bind(account)
    .bind(tx_id)
    .bind(role)
    .bind(block_height)
    .bind(timestamp)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get transactions for account
pub async fn get_account_transactions(
    pool: &PgPool,
    account: &str,
    offset: i64,
    limit: i64,
) -> Result<Vec<AccountTransactionRow>> {
    sqlx::query_as::<_, AccountTransactionRow>(
        r#"
        SELECT * FROM account_transactions
        WHERE account = $1
        ORDER BY timestamp DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(account)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

/// Count transactions for account
pub async fn count_account_transactions(pool: &PgPool, account: &str) -> Result<i64> {
    let row = sqlx::query_as::<_, CountRow>(
        "SELECT COUNT(*) as count FROM account_transactions WHERE account = $1",
    )
    .bind(account)
    .fetch_one(pool)
    .await?;

    Ok(row.value())
}

/// Search accounts by address prefix
pub async fn search_accounts(pool: &PgPool, query: &str, limit: i64) -> Result<Vec<AccountRow>> {
    let pattern = format!("{}%", query);
    sqlx::query_as::<_, AccountRow>(
        "SELECT * FROM accounts WHERE address LIKE $1 ORDER BY tx_count DESC LIMIT $2",
    )
    .bind(&pattern)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

/// Get top accounts by ATLAS balance
pub async fn get_top_atlas_holders(pool: &PgPool, limit: i64) -> Result<Vec<AccountRow>> {
    sqlx::query_as::<_, AccountRow>(
        "SELECT * FROM accounts ORDER BY atlas_balance DESC LIMIT $1",
    )
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

/// Get top accounts by SHRUG balance
pub async fn get_top_shrug_holders(pool: &PgPool, limit: i64) -> Result<Vec<AccountRow>> {
    sqlx::query_as::<_, AccountRow>(
        "SELECT * FROM accounts ORDER BY shrug_balance DESC LIMIT $1",
    )
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

/// Check if account exists
pub async fn account_exists(pool: &PgPool, address: &str) -> Result<bool> {
    let row = sqlx::query_as::<_, CountRow>(
        "SELECT COUNT(*) as count FROM accounts WHERE address = $1",
    )
    .bind(address)
    .fetch_one(pool)
    .await?;

    Ok(row.value() > 0)
}
