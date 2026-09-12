use crate::{ReceiptRow, Result, TxDetailRow, TxRow};
use sqlx::{PgConnection, PgPool};

/// Columns of a transaction list row.
pub const TX_COLS: &str = "hash, block_hash, height, tx_index, kind, fee::text AS fee, timestamp_ms, has_bundle, program_id, validator, amount::text AS amount, asset_index";

/// Every column: `TX_COLS` plus the bundle and the action fields.
pub const TX_DETAIL_COLS: &str = "hash, block_hash, height, tx_index, kind, fee::text AS fee, timestamp_ms, has_bundle, program_id, validator, amount::text AS amount, asset_index,
    chain_id, anchor, nullifier_1, nullifier_2, commitment_1, commitment_2, burn::text AS burn, asset, bundle_time, proof_len, envelope_len_1, envelope_len_2,
    words_len, call_proof_len, input_envelope_len, cm, registered, action_nonce, attestation_len, recipient, note_time,
    relayer_fee::text AS relayer_fee, to_chain, bridge_to, asset_bundle";

/// The public fields of one bundle, as the indexer stores them.
#[derive(Debug, Clone, Default)]
pub struct NewBundle {
    pub anchor: String,
    pub nullifiers: [String; 2],
    pub commitments: [String; 2],
    pub fee: String,
    pub burn: String,
    pub asset: i64,
    pub time: i64,
    pub proof_len: i64,
    pub envelope_len: [i64; 2],
}

#[derive(Debug, Clone, Default)]
pub struct NewTx<'a> {
    pub hash: &'a str,
    pub block_hash: &'a str,
    pub height: i64,
    pub tx_index: i32,
    pub chain_id: i64,
    pub timestamp_ms: i64,
    pub kind: &'a str,
    pub bundle: Option<NewBundle>,
    pub program_id: Option<&'a str>,
    pub words_len: Option<i64>,
    pub call_proof_len: Option<i64>,
    pub input_envelope_len: Option<i64>,
    pub amount: Option<String>,
    pub cm: Option<&'a str>,
    pub validator: Option<&'a str>,
    pub registered: Option<bool>,
    pub action_nonce: Option<i64>,
    pub attestation_len: Option<i64>,
    pub recipient: Option<&'a str>,
    pub note_time: Option<i64>,
    pub asset_index: Option<i64>,
    pub relayer_fee: Option<String>,
    pub to_chain: Option<i32>,
    pub bridge_to: Option<&'a str>,
    pub asset_bundle: Option<NewBundle>,
}

fn bundle_json(b: &NewBundle) -> serde_json::Value {
    serde_json::json!({
        "anchor": b.anchor, "nullifiers": b.nullifiers, "commitments": b.commitments,
        "fee": b.fee, "burn": b.burn, "asset": b.asset, "time": b.time,
        "proof_len": b.proof_len, "envelope_len": b.envelope_len,
    })
}

/// Insert the transaction and every nullifier its bundles published.
pub async fn insert_transaction(conn: &mut PgConnection, t: &NewTx<'_>) -> Result<()> {
    let b = t.bundle.as_ref();
    let fee = b.map(|b| b.fee.as_str()).unwrap_or("0");
    sqlx::query(
        "INSERT INTO transactions (hash, block_hash, height, tx_index, chain_id, timestamp_ms, kind, fee,
             has_bundle, anchor, nullifier_1, nullifier_2, commitment_1, commitment_2, burn, asset, bundle_time,
             proof_len, envelope_len_1, envelope_len_2,
             program_id, words_len, call_proof_len, input_envelope_len, amount, cm, validator, registered,
             action_nonce, attestation_len, recipient, note_time, asset_index, relayer_fee, to_chain, bridge_to,
             asset_bundle)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8::numeric,
             $9, $10, $11, $12, $13, $14, $15::numeric, $16, $17, $18, $19, $20,
             $21, $22, $23, $24, $25::numeric, $26, $27, $28, $29, $30, $31, $32, $33, $34::numeric, $35, $36, $37)",
    )
    .bind(t.hash)
    .bind(t.block_hash)
    .bind(t.height)
    .bind(t.tx_index)
    .bind(t.chain_id)
    .bind(t.timestamp_ms)
    .bind(t.kind)
    .bind(fee)
    .bind(b.is_some())
    .bind(b.map(|b| b.anchor.as_str()))
    .bind(b.map(|b| b.nullifiers[0].as_str()))
    .bind(b.map(|b| b.nullifiers[1].as_str()))
    .bind(b.map(|b| b.commitments[0].as_str()))
    .bind(b.map(|b| b.commitments[1].as_str()))
    .bind(b.map(|b| b.burn.as_str()))
    .bind(b.map(|b| b.asset))
    .bind(b.map(|b| b.time))
    .bind(b.map(|b| b.proof_len))
    .bind(b.map(|b| b.envelope_len[0]))
    .bind(b.map(|b| b.envelope_len[1]))
    .bind(t.program_id)
    .bind(t.words_len)
    .bind(t.call_proof_len)
    .bind(t.input_envelope_len)
    .bind(t.amount.as_deref())
    .bind(t.cm)
    .bind(t.validator)
    .bind(t.registered)
    .bind(t.action_nonce)
    .bind(t.attestation_len)
    .bind(t.recipient)
    .bind(t.note_time)
    .bind(t.asset_index)
    .bind(t.relayer_fee.as_deref())
    .bind(t.to_chain)
    .bind(t.bridge_to)
    .bind(t.asset_bundle.as_ref().map(bundle_json))
    .execute(&mut *conn)
    .await?;

    for bundle in t.bundle.iter().chain(t.asset_bundle.iter()) {
        for nf in &bundle.nullifiers {
            // A dummy input still publishes a nullifier, and the chain rejects a repeat, so a
            // conflict here can only be a re-index of the same block.
            sqlx::query(
                "INSERT INTO nullifiers (nullifier, tx_hash, height, tx_index) VALUES ($1, $2, $3, $4)
                 ON CONFLICT (nullifier) DO NOTHING",
            )
            .bind(nf)
            .bind(t.hash)
            .bind(t.height)
            .bind(t.tx_index)
            .execute(&mut *conn)
            .await?;
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn insert_receipt(
    conn: &mut PgConnection,
    tx_hash: &str,
    program: &str,
    tier: i32,
    outputs: &[i64],
    height: i64,
    tx_index: i32,
    h_in: &str,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO receipts (tx_hash, program, tier, outputs, height, tx_index, h_in)
         VALUES ($1, $2, $3, $4, $5, $6, $7) ON CONFLICT (tx_hash) DO NOTHING",
    )
    .bind(tx_hash)
    .bind(program)
    .bind(tier)
    .bind(outputs)
    .bind(height)
    .bind(tx_index)
    .bind(h_in)
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn get_transaction(pool: &PgPool, hash: &str) -> Result<Option<TxDetailRow>> {
    let sql = format!("SELECT {TX_DETAIL_COLS} FROM transactions WHERE hash = $1");
    Ok(sqlx::query_as::<_, TxDetailRow>(&sql)
        .bind(hash)
        .fetch_optional(pool)
        .await?)
}

pub async fn get_transaction_summary(pool: &PgPool, hash: &str) -> Result<Option<TxRow>> {
    let sql = format!("SELECT {TX_COLS} FROM transactions WHERE hash = $1");
    Ok(sqlx::query_as::<_, TxRow>(&sql)
        .bind(hash)
        .fetch_optional(pool)
        .await?)
}

pub async fn get_receipt(pool: &PgPool, tx_hash: &str) -> Result<Option<ReceiptRow>> {
    Ok(sqlx::query_as::<_, ReceiptRow>(
        "SELECT tx_hash, program, tier, outputs, height, tx_index, h_in FROM receipts WHERE tx_hash = $1",
    )
    .bind(tx_hash)
    .fetch_optional(pool)
    .await?)
}

pub async fn get_transactions_for_block(pool: &PgPool, block_hash: &str) -> Result<Vec<TxRow>> {
    let sql =
        format!("SELECT {TX_COLS} FROM transactions WHERE block_hash = $1 ORDER BY tx_index ASC");
    Ok(sqlx::query_as::<_, TxRow>(&sql)
        .bind(block_hash)
        .fetch_all(pool)
        .await?)
}

pub async fn get_latest_transactions(pool: &PgPool, limit: i64) -> Result<Vec<TxRow>> {
    let sql =
        format!("SELECT {TX_COLS} FROM transactions ORDER BY height DESC, tx_index DESC LIMIT $1");
    Ok(sqlx::query_as::<_, TxRow>(&sql)
        .bind(limit)
        .fetch_all(pool)
        .await?)
}

/// Filters of a transaction list (`GET /transactions`).
#[derive(Debug, Clone, Default)]
pub struct TxFilter<'a> {
    pub kind: Option<&'a str>,
    pub height: Option<i64>,
    pub validator: Option<&'a str>,
    pub program: Option<&'a str>,
}

const FILTER_WHERE: &str = "($1::text IS NULL OR kind = $1) AND ($2::bigint IS NULL OR height = $2)
         AND ($3::text IS NULL OR validator = $3) AND ($4::text IS NULL OR program_id = $4)";

pub async fn list_transactions(
    pool: &PgPool,
    offset: i64,
    limit: i64,
    f: &TxFilter<'_>,
) -> Result<Vec<TxRow>> {
    let sql = format!(
        "SELECT {TX_COLS} FROM transactions WHERE {FILTER_WHERE}
         ORDER BY height DESC, tx_index DESC LIMIT $5 OFFSET $6"
    );
    Ok(sqlx::query_as::<_, TxRow>(&sql)
        .bind(f.kind)
        .bind(f.height)
        .bind(f.validator)
        .bind(f.program)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?)
}

pub async fn count_transactions(pool: &PgPool, f: &TxFilter<'_>) -> Result<i64> {
    let sql = format!("SELECT COUNT(*) FROM transactions WHERE {FILTER_WHERE}");
    Ok(sqlx::query_scalar(&sql)
        .bind(f.kind)
        .bind(f.height)
        .bind(f.validator)
        .bind(f.program)
        .fetch_one(pool)
        .await?)
}

pub async fn list_program_calls(pool: &PgPool, program: &str, limit: i64) -> Result<Vec<TxRow>> {
    let sql = format!(
        "SELECT {TX_COLS} FROM transactions WHERE kind = 'call' AND program_id = $1 ORDER BY height DESC, tx_index DESC LIMIT $2"
    );
    Ok(sqlx::query_as::<_, TxRow>(&sql)
        .bind(program)
        .bind(limit)
        .fetch_all(pool)
        .await?)
}

/// The transaction whose bundle (or mint) created the note with commitment `cm`.
pub async fn find_transaction_by_commitment(pool: &PgPool, cm: &str) -> Result<Option<TxRow>> {
    let sql = format!(
        "SELECT {TX_COLS} FROM transactions WHERE commitment_1 = $1 OR commitment_2 = $1 OR cm = $1
         OR (asset_bundle IS NOT NULL AND asset_bundle->'commitments' ? $1) LIMIT 1"
    );
    Ok(sqlx::query_as::<_, TxRow>(&sql)
        .bind(cm)
        .fetch_optional(pool)
        .await?)
}
