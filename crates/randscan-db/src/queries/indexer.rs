use crate::{IndexerStateRow, Result};
use sqlx::{PgConnection, PgPool};

pub async fn get_indexer_state(pool: &PgPool) -> Result<IndexerStateRow> {
    Ok(sqlx::query_as::<_, IndexerStateRow>(
        "SELECT next_height, last_indexed_hash, is_syncing FROM indexer_state WHERE id = 1",
    )
    .fetch_one(pool)
    .await?)
}

pub async fn set_next_height(
    conn: &mut PgConnection,
    next_height: i64,
    last_hash: Option<&str>,
) -> Result<()> {
    sqlx::query(
        "UPDATE indexer_state SET next_height = $1, last_indexed_hash = $2, updated_at = NOW() WHERE id = 1",
    )
    .bind(next_height)
    .bind(last_hash)
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn set_syncing(pool: &PgPool, syncing: bool) -> Result<()> {
    sqlx::query("UPDATE indexer_state SET is_syncing = $1, updated_at = NOW() WHERE id = 1")
        .bind(syncing)
        .execute(pool)
        .await?;
    Ok(())
}
