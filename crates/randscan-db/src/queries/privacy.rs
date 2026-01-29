//! Privacy tracking queries (nullifiers and commitments)

use crate::{CommitmentRow, CountRow, NullifierRow, Result};
use sqlx::PgPool;

/// Insert nullifier
pub async fn insert_nullifier(pool: &PgPool, nullifier: &str, tx_id: &str) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO nullifiers (nullifier, tx_id)
        VALUES ($1, $2)
        ON CONFLICT (nullifier) DO NOTHING
        "#,
    )
    .bind(nullifier)
    .bind(tx_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Check if nullifier exists (was spent)
pub async fn nullifier_exists(pool: &PgPool, nullifier: &str) -> Result<bool> {
    let row = sqlx::query_as::<_, CountRow>(
        "SELECT COUNT(*) as count FROM nullifiers WHERE nullifier = $1",
    )
    .bind(nullifier)
    .fetch_one(pool)
    .await?;

    Ok(row.value() > 0)
}

/// Get nullifier by value
pub async fn get_nullifier(pool: &PgPool, nullifier: &str) -> Result<NullifierRow> {
    sqlx::query_as::<_, NullifierRow>("SELECT * FROM nullifiers WHERE nullifier = $1")
        .bind(nullifier)
        .fetch_one(pool)
        .await
        .map_err(Into::into)
}

/// Get nullifiers for transaction
pub async fn get_nullifiers_for_tx(pool: &PgPool, tx_id: &str) -> Result<Vec<NullifierRow>> {
    sqlx::query_as::<_, NullifierRow>("SELECT * FROM nullifiers WHERE tx_id = $1")
        .bind(tx_id)
        .fetch_all(pool)
        .await
        .map_err(Into::into)
}

/// Count total nullifiers
pub async fn count_nullifiers(pool: &PgPool) -> Result<i64> {
    let row = sqlx::query_as::<_, CountRow>("SELECT COUNT(*) as count FROM nullifiers")
        .fetch_one(pool)
        .await?;

    Ok(row.value())
}

/// Insert commitment
pub async fn insert_commitment(pool: &PgPool, commitment: &str, tx_id: &str) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO commitments (commitment, tx_id, spent)
        VALUES ($1, $2, FALSE)
        ON CONFLICT (commitment) DO NOTHING
        "#,
    )
    .bind(commitment)
    .bind(tx_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Mark commitment as spent
pub async fn mark_commitment_spent(
    pool: &PgPool,
    commitment: &str,
    spent_tx_id: &str,
) -> Result<()> {
    sqlx::query(
        "UPDATE commitments SET spent = TRUE, spent_tx_id = $2 WHERE commitment = $1",
    )
    .bind(commitment)
    .bind(spent_tx_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Check if commitment exists
pub async fn commitment_exists(pool: &PgPool, commitment: &str) -> Result<bool> {
    let row = sqlx::query_as::<_, CountRow>(
        "SELECT COUNT(*) as count FROM commitments WHERE commitment = $1",
    )
    .bind(commitment)
    .fetch_one(pool)
    .await?;

    Ok(row.value() > 0)
}

/// Get commitment by value
pub async fn get_commitment(pool: &PgPool, commitment: &str) -> Result<CommitmentRow> {
    sqlx::query_as::<_, CommitmentRow>("SELECT * FROM commitments WHERE commitment = $1")
        .bind(commitment)
        .fetch_one(pool)
        .await
        .map_err(Into::into)
}

/// Get commitments for transaction
pub async fn get_commitments_for_tx(pool: &PgPool, tx_id: &str) -> Result<Vec<CommitmentRow>> {
    sqlx::query_as::<_, CommitmentRow>("SELECT * FROM commitments WHERE tx_id = $1")
        .bind(tx_id)
        .fetch_all(pool)
        .await
        .map_err(Into::into)
}

/// Count total commitments
pub async fn count_commitments(pool: &PgPool) -> Result<i64> {
    let row = sqlx::query_as::<_, CountRow>("SELECT COUNT(*) as count FROM commitments")
        .fetch_one(pool)
        .await?;

    Ok(row.value())
}

/// Count unspent commitments
pub async fn count_unspent_commitments(pool: &PgPool) -> Result<i64> {
    let row = sqlx::query_as::<_, CountRow>(
        "SELECT COUNT(*) as count FROM commitments WHERE spent = FALSE",
    )
    .fetch_one(pool)
    .await?;

    Ok(row.value())
}

/// Get recent nullifiers
pub async fn get_recent_nullifiers(pool: &PgPool, limit: i64) -> Result<Vec<NullifierRow>> {
    sqlx::query_as::<_, NullifierRow>(
        "SELECT * FROM nullifiers ORDER BY created_at DESC LIMIT $1",
    )
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

/// Get recent commitments
pub async fn get_recent_commitments(pool: &PgPool, limit: i64) -> Result<Vec<CommitmentRow>> {
    sqlx::query_as::<_, CommitmentRow>(
        "SELECT * FROM commitments ORDER BY created_at DESC LIMIT $1",
    )
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}
