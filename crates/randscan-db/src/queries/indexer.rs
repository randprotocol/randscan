//! Indexer state queries

use crate::{IndexerStateRow, Result};
use chrono::Utc;
use sqlx::PgPool;

/// Get indexer state
pub async fn get_indexer_state(pool: &PgPool) -> Result<IndexerStateRow> {
    sqlx::query_as::<_, IndexerStateRow>("SELECT * FROM indexer_state WHERE id = 1")
        .fetch_one(pool)
        .await
        .map_err(Into::into)
}

/// Update last indexed height
pub async fn update_last_indexed(
    pool: &PgPool,
    height: i64,
    block_id: &str,
) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE indexer_state SET
            last_indexed_height = $1,
            last_indexed_block_id = $2
        WHERE id = 1
        "#,
    )
    .bind(height)
    .bind(block_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Update last finalized height
pub async fn update_last_finalized(pool: &PgPool, height: i64) -> Result<()> {
    sqlx::query("UPDATE indexer_state SET last_finalized_height = $1 WHERE id = 1")
        .bind(height)
        .execute(pool)
        .await?;

    Ok(())
}

/// Set syncing status
pub async fn set_syncing(pool: &PgPool, is_syncing: bool) -> Result<()> {
    if is_syncing {
        sqlx::query(
            "UPDATE indexer_state SET is_syncing = TRUE, sync_started_at = $1 WHERE id = 1",
        )
        .bind(Utc::now())
        .execute(pool)
        .await?;
    } else {
        sqlx::query(
            "UPDATE indexer_state SET is_syncing = FALSE, sync_started_at = NULL WHERE id = 1",
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}

/// Get last indexed height
pub async fn get_last_indexed_height(pool: &PgPool) -> Result<i64> {
    let state = get_indexer_state(pool).await?;
    Ok(state.last_indexed_height)
}

/// Check if currently syncing
pub async fn is_syncing(pool: &PgPool) -> Result<bool> {
    let state = get_indexer_state(pool).await?;
    Ok(state.is_syncing)
}
