use crate::{ReceiverRow, Result};
use sqlx::{PgConnection, PgPool};

pub async fn insert_receiver(conn: &mut PgConnection, row: &ReceiverRow) -> Result<()> {
    sqlx::query(
        "INSERT INTO receivers (id, version, pk, kem_ek, signing_key, signature, tx_hash, height)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) ON CONFLICT (id, version) DO NOTHING",
    )
    .bind(&row.id)
    .bind(row.version)
    .bind(&row.pk)
    .bind(&row.kem_ek)
    .bind(&row.signing_key)
    .bind(&row.signature)
    .bind(&row.tx_hash)
    .bind(row.height)
    .execute(conn)
    .await?;
    Ok(())
}

/// The current record: the highest version published for `id`.
pub async fn get_receiver(pool: &PgPool, id: &str) -> Result<Option<ReceiverRow>> {
    Ok(sqlx::query_as::<_, ReceiverRow>(
        "SELECT id, version, pk, kem_ek, signing_key, signature, tx_hash, height
         FROM receivers WHERE id = $1 ORDER BY version DESC LIMIT 1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?)
}

/// Every published version of `id`, newest first.
pub async fn receiver_history(pool: &PgPool, id: &str) -> Result<Vec<ReceiverRow>> {
    Ok(sqlx::query_as::<_, ReceiverRow>(
        "SELECT id, version, pk, kem_ek, signing_key, signature, tx_hash, height
         FROM receivers WHERE id = $1 ORDER BY version DESC",
    )
    .bind(id)
    .fetch_all(pool)
    .await?)
}
