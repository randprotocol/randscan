//! Token queries

use crate::{CountRow, DbError, Result, TokenAccountRow, TokenMintRow};
use sqlx::PgPool;

/// Insert or update token mint
pub async fn upsert_token_mint(
    pool: &PgPool,
    mint_address: &str,
    symbol: &str,
    name: &str,
    decimals: i16,
    total_supply: i64,
    circulating_supply: i64,
    burned: i64,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO token_mints (
            mint_address, symbol, name, decimals, total_supply,
            circulating_supply, burned
        ) VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (mint_address) DO UPDATE SET
            total_supply = EXCLUDED.total_supply,
            circulating_supply = EXCLUDED.circulating_supply,
            burned = EXCLUDED.burned
        "#,
    )
    .bind(mint_address)
    .bind(symbol)
    .bind(name)
    .bind(decimals)
    .bind(total_supply)
    .bind(circulating_supply)
    .bind(burned)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get token mint by address
pub async fn get_token_mint(pool: &PgPool, mint_address: &str) -> Result<TokenMintRow> {
    sqlx::query_as::<_, TokenMintRow>("SELECT * FROM token_mints WHERE mint_address = $1")
        .bind(mint_address)
        .fetch_one(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                DbError::NotFound(format!("Token {} not found", mint_address))
            }
            _ => e.into(),
        })
}

/// Get all token mints
pub async fn get_token_mints(pool: &PgPool, offset: i64, limit: i64) -> Result<Vec<TokenMintRow>> {
    sqlx::query_as::<_, TokenMintRow>(
        "SELECT * FROM token_mints ORDER BY holder_count DESC LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

/// Count token mints
pub async fn count_token_mints(pool: &PgPool) -> Result<i64> {
    let row = sqlx::query_as::<_, CountRow>("SELECT COUNT(*) as count FROM token_mints")
        .fetch_one(pool)
        .await?;

    Ok(row.value())
}

/// Update token mint supply
pub async fn update_token_supply(
    pool: &PgPool,
    mint_address: &str,
    total_supply: i64,
    circulating_supply: i64,
    burned: i64,
) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE token_mints SET
            total_supply = $2,
            circulating_supply = $3,
            burned = $4
        WHERE mint_address = $1
        "#,
    )
    .bind(mint_address)
    .bind(total_supply)
    .bind(circulating_supply)
    .bind(burned)
    .execute(pool)
    .await?;

    Ok(())
}

/// Increment token holder count
pub async fn increment_holder_count(pool: &PgPool, mint_address: &str) -> Result<()> {
    sqlx::query("UPDATE token_mints SET holder_count = holder_count + 1 WHERE mint_address = $1")
        .bind(mint_address)
        .execute(pool)
        .await?;

    Ok(())
}

/// Increment token tx count
pub async fn increment_token_tx_count(pool: &PgPool, mint_address: &str) -> Result<()> {
    sqlx::query("UPDATE token_mints SET tx_count = tx_count + 1 WHERE mint_address = $1")
        .bind(mint_address)
        .execute(pool)
        .await?;

    Ok(())
}

/// Insert or update token account
pub async fn upsert_token_account(
    pool: &PgPool,
    account_address: &str,
    owner: &str,
    mint: &str,
    balance: i64,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO token_accounts (account_address, owner, mint, balance)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (owner, mint) DO UPDATE SET
            balance = EXCLUDED.balance,
            account_address = EXCLUDED.account_address
        "#,
    )
    .bind(account_address)
    .bind(owner)
    .bind(mint)
    .bind(balance)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get token accounts for owner
pub async fn get_token_accounts_for_owner(
    pool: &PgPool,
    owner: &str,
) -> Result<Vec<TokenAccountRow>> {
    sqlx::query_as::<_, TokenAccountRow>("SELECT * FROM token_accounts WHERE owner = $1")
        .bind(owner)
        .fetch_all(pool)
        .await
        .map_err(Into::into)
}

/// Get token accounts for mint
pub async fn get_token_accounts_for_mint(
    pool: &PgPool,
    mint: &str,
    offset: i64,
    limit: i64,
) -> Result<Vec<TokenAccountRow>> {
    sqlx::query_as::<_, TokenAccountRow>(
        "SELECT * FROM token_accounts WHERE mint = $1 ORDER BY balance DESC LIMIT $2 OFFSET $3",
    )
    .bind(mint)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

/// Count token accounts for mint (holder count)
pub async fn count_token_holders(pool: &PgPool, mint: &str) -> Result<i64> {
    let row = sqlx::query_as::<_, CountRow>(
        "SELECT COUNT(*) as count FROM token_accounts WHERE mint = $1 AND balance > 0",
    )
    .bind(mint)
    .fetch_one(pool)
    .await?;

    Ok(row.value())
}

/// Update token account balance
pub async fn update_token_balance(
    pool: &PgPool,
    owner: &str,
    mint: &str,
    balance_delta: i64,
) -> Result<()> {
    sqlx::query(
        "UPDATE token_accounts SET balance = balance + $3 WHERE owner = $1 AND mint = $2",
    )
    .bind(owner)
    .bind(mint)
    .bind(balance_delta)
    .execute(pool)
    .await?;

    Ok(())
}

/// Initialize ATLAS and SHRUG tokens
pub async fn initialize_native_tokens(pool: &PgPool) -> Result<()> {
    // ATLAS token
    sqlx::query(
        r#"
        INSERT INTO token_mints (mint_address, symbol, name, decimals, total_supply, circulating_supply, burned)
        VALUES ('ATLAS', 'ATLAS', 'ATLAS Token', 9, 0, 0, 0)
        ON CONFLICT (mint_address) DO NOTHING
        "#,
    )
    .execute(pool)
    .await?;

    // SHRUG token
    sqlx::query(
        r#"
        INSERT INTO token_mints (mint_address, symbol, name, decimals, total_supply, circulating_supply, burned)
        VALUES ('SHRUG', 'SHRUG', 'SHRUG Token', 9, 0, 0, 0)
        ON CONFLICT (mint_address) DO NOTHING
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Search tokens by symbol
pub async fn search_tokens(pool: &PgPool, query: &str, limit: i64) -> Result<Vec<TokenMintRow>> {
    let pattern = format!("%{}%", query.to_uppercase());
    sqlx::query_as::<_, TokenMintRow>(
        r#"
        SELECT * FROM token_mints
        WHERE UPPER(symbol) LIKE $1 OR UPPER(name) LIKE $1
        ORDER BY holder_count DESC
        LIMIT $2
        "#,
    )
    .bind(&pattern)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}
