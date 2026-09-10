use crate::{AccountRow, Result};
use sqlx::{PgConnection, PgPool};

/// Insert or refresh an account from the node's view (balance and nonce are authoritative).
pub async fn upsert_account(
    conn: &mut PgConnection,
    address: &str,
    balance: &str,
    nonce: i64,
    seen_height: i64,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO accounts (address, balance, nonce, first_seen_height, last_seen_height)
         VALUES ($1, $2::numeric, $3, $4, $4)
         ON CONFLICT (address) DO UPDATE SET
            balance = EXCLUDED.balance,
            nonce = EXCLUDED.nonce,
            first_seen_height = LEAST(accounts.first_seen_height, EXCLUDED.first_seen_height),
            last_seen_height = GREATEST(accounts.last_seen_height, EXCLUDED.last_seen_height),
            updated_at = NOW()",
    )
    .bind(address)
    .bind(balance)
    .bind(nonce)
    .bind(seen_height)
    .execute(conn)
    .await?;
    Ok(())
}

/// Recompute an account's transaction count from `account_transactions`.
pub async fn refresh_account_tx_count(conn: &mut PgConnection, address: &str) -> Result<()> {
    sqlx::query(
        "UPDATE accounts SET tx_count = (SELECT COUNT(DISTINCT tx_hash) FROM account_transactions WHERE account = $1)
         WHERE address = $1",
    )
    .bind(address)
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn get_account(pool: &PgPool, address: &str) -> Result<Option<AccountRow>> {
    Ok(sqlx::query_as::<_, AccountRow>(
        "SELECT address, balance::text AS balance, nonce, tx_count, first_seen_height, last_seen_height FROM accounts WHERE address = $1",
    )
    .bind(address)
    .fetch_optional(pool)
    .await?)
}

pub async fn count_accounts(pool: &PgPool) -> Result<i64> {
    Ok(sqlx::query_scalar("SELECT COUNT(*) FROM accounts")
        .fetch_one(pool)
        .await?)
}

pub async fn total_supply(pool: &PgPool) -> Result<String> {
    Ok(
        sqlx::query_scalar("SELECT COALESCE(SUM(balance), 0)::text FROM accounts")
            .fetch_one(pool)
            .await?,
    )
}
