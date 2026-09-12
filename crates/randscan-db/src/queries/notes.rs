//! The commitment tree (leaf by leaf) and the nullifier set.

use crate::{NoteRow, NullifierRow, Result};
use sqlx::{PgConnection, PgPool};

const NOTE_COLS: &str = "leaf_index, cm, height, tx_hash";

/// One tree leaf as the node serves it: index, commitment, height and the envelope (hex fields).
pub struct NewNote {
    pub leaf_index: i64,
    pub cm: String,
    pub height: i64,
    pub envelope: Option<serde_json::Value>,
}

/// Store a page of tree leaves. The tree is append-only, so a re-read page carries the same
/// rows: an existing leaf only gains its envelope when it had none (the backfill after
/// migration 006). `tx_hash` is resolved from the indexed transactions that carried the
/// commitment on the wire.
pub async fn insert_notes(conn: &mut PgConnection, rows: &[NewNote]) -> Result<()> {
    for n in rows {
        sqlx::query(
            "INSERT INTO notes (leaf_index, cm, height, tx_hash, envelope)
             VALUES ($1, $2, $3,
                (SELECT hash FROM transactions WHERE commitment_1 = $2 OR commitment_2 = $2 OR cm = $2
                    OR (asset_bundle IS NOT NULL AND asset_bundle->'commitments' ? $2) LIMIT 1), $4)
             ON CONFLICT (leaf_index) DO UPDATE SET envelope = COALESCE(notes.envelope, EXCLUDED.envelope)",
        )
        .bind(n.leaf_index)
        .bind(&n.cm)
        .bind(n.height)
        .bind(&n.envelope)
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

/// A leaf with its envelope, for the browser-side opener.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct NoteEnvelopeRow {
    pub leaf_index: i64,
    pub cm: String,
    pub height: i64,
    pub tx_hash: Option<String>,
    pub envelope: Option<serde_json::Value>,
}

/// The leaves whose commitments the transaction `hash` published (bundle outputs, a mint's
/// note, a burn's asset-bundle outputs), with their envelopes.
pub async fn get_note_envelopes_for_tx(pool: &PgPool, hash: &str) -> Result<Vec<NoteEnvelopeRow>> {
    Ok(sqlx::query_as::<_, NoteEnvelopeRow>(
        "SELECT n.leaf_index, n.cm, n.height, n.tx_hash, n.envelope FROM notes n
         WHERE n.tx_hash = $1
            OR n.cm IN (SELECT commitment_1 FROM transactions WHERE hash = $1 AND commitment_1 IS NOT NULL
                        UNION SELECT commitment_2 FROM transactions WHERE hash = $1 AND commitment_2 IS NOT NULL
                        UNION SELECT cm FROM transactions WHERE hash = $1 AND cm IS NOT NULL)
         ORDER BY n.leaf_index",
    )
    .bind(hash)
    .fetch_all(pool)
    .await?)
}

/// A page of leaves with envelopes from `from_leaf` upwards (for a history scan in the browser).
pub async fn list_note_envelopes(pool: &PgPool, from_leaf: i64, limit: i64) -> Result<Vec<NoteEnvelopeRow>> {
    Ok(sqlx::query_as::<_, NoteEnvelopeRow>(
        "SELECT leaf_index, cm, height, tx_hash, envelope FROM notes WHERE leaf_index >= $1 ORDER BY leaf_index LIMIT $2",
    )
    .bind(from_leaf)
    .bind(limit)
    .fetch_all(pool)
    .await?)
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
