//! The commitment tree (leaf by leaf) and the nullifier set.

use crate::{NoteRow, NullifierRow, Result};
use sqlx::{PgConnection, PgPool};

const NOTE_COLS: &str = "leaf_index, cm, height, tx_hash";

/// Store a page of tree leaves. A leaf that is already stored is left alone (the tree is
/// append-only, so a re-read page carries the same rows). `tx_hash` is resolved from the
/// indexed transactions that carried the commitment on the wire.
pub async fn insert_notes(conn: &mut PgConnection, rows: &[(i64, String, i64)]) -> Result<()> {
    for (leaf_index, cm, height) in rows {
        sqlx::query(
            "INSERT INTO notes (leaf_index, cm, height, tx_hash)
             VALUES ($1, $2, $3,
                (SELECT hash FROM transactions WHERE commitment_1 = $2 OR commitment_2 = $2 OR cm = $2
                    OR (asset_bundle IS NOT NULL AND asset_bundle->'commitments' ? $2) LIMIT 1))
             ON CONFLICT (leaf_index) DO NOTHING",
        )
        .bind(leaf_index)
        .bind(cm)
        .bind(height)
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

/// Link notes of `height` to the transactions that created them (for leaves fetched before the
/// block was indexed, e.g. after a rewind).
pub async fn link_notes_at(conn: &mut PgConnection, height: i64) -> Result<()> {
    sqlx::query(
        "UPDATE notes n SET tx_hash = t.hash FROM transactions t
         WHERE n.height = $1 AND n.tx_hash IS NULL AND t.height = $1
           AND (t.commitment_1 = n.cm OR t.commitment_2 = n.cm OR t.cm = n.cm
                OR (t.asset_bundle IS NOT NULL AND t.asset_bundle->'commitments' ? n.cm))",
    )
    .bind(height)
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn list_notes(pool: &PgPool, offset: i64, limit: i64) -> Result<Vec<NoteRow>> {
    let sql = format!("SELECT {NOTE_COLS} FROM notes ORDER BY leaf_index DESC LIMIT $1 OFFSET $2");
    Ok(sqlx::query_as::<_, NoteRow>(&sql)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?)
}

pub async fn count_notes(pool: &PgPool) -> Result<i64> {
    Ok(sqlx::query_scalar("SELECT COUNT(*) FROM notes")
        .fetch_one(pool)
        .await?)
}

pub async fn get_note_by_cm(pool: &PgPool, cm: &str) -> Result<Option<NoteRow>> {
    let sql = format!("SELECT {NOTE_COLS} FROM notes WHERE cm = $1");
    Ok(sqlx::query_as::<_, NoteRow>(&sql)
        .bind(cm)
        .fetch_optional(pool)
        .await?)
}

pub async fn get_note_by_index(pool: &PgPool, leaf_index: i64) -> Result<Option<NoteRow>> {
    let sql = format!("SELECT {NOTE_COLS} FROM notes WHERE leaf_index = $1");
    Ok(sqlx::query_as::<_, NoteRow>(&sql)
        .bind(leaf_index)
        .fetch_optional(pool)
        .await?)
}

/// Leaves at and above `height` (dropped on a rewind, since the tree past a lost block is gone).
pub async fn delete_notes_from(conn: &mut PgConnection, height: i64) -> Result<u64> {
    Ok(sqlx::query("DELETE FROM notes WHERE height >= $1")
        .bind(height)
        .execute(conn)
        .await?
        .rows_affected())
}

pub async fn get_nullifier(pool: &PgPool, nullifier: &str) -> Result<Option<NullifierRow>> {
    Ok(sqlx::query_as::<_, NullifierRow>(
        "SELECT nullifier, tx_hash, height, tx_index FROM nullifiers WHERE nullifier = $1",
    )
    .bind(nullifier)
    .fetch_optional(pool)
    .await?)
}

pub async fn count_nullifiers(pool: &PgPool) -> Result<i64> {
    Ok(sqlx::query_scalar("SELECT COUNT(*) FROM nullifiers")
        .fetch_one(pool)
        .await?)
}

/// The leaf index the next fetch must start at after leaves were deleted: one past the highest
/// leaf still stored (the tree is contiguous from 0).
pub async fn next_leaf_after_rewind(conn: &mut PgConnection) -> Result<i64> {
    Ok(
        sqlx::query_scalar("SELECT COALESCE(MAX(leaf_index) + 1, 0) FROM notes")
            .fetch_one(conn)
            .await?,
    )
}
