use crate::{AccountTxRow, ReceiptRow, Result, TxRow};
use sqlx::{PgConnection, PgPool};

/// Columns of a transaction row. `attestation` (up to 32 KiB of hex) is deliberately left out of
/// list queries; `get_attestation` fetches it for the detail view.
pub const TX_COLS: &str = "hash, block_hash, height, tx_index, sender, nonce, fee::text AS fee, kind, chain_id, timestamp_ms, to_address, amount::text AS amount, program_id, base_pc, words_len, proof_len, recipients, asset, bridge_amount::text AS bridge_amount, to_chain, bridge_to, bridge_fee::text AS bridge_fee";

/// `TX_COLS` with every column prefixed by a table alias (for joins).
pub fn tx_cols_qualified(alias: &str) -> String {
    TX_COLS
        .split(", ")
        .map(|c| format!("{alias}.{c}"))
        .collect::<Vec<_>>()
        .join(", ")
}

pub struct NewTx<'a> {
    pub hash: &'a str,
    pub block_hash: &'a str,
    pub height: i64,
    pub tx_index: i32,
    pub sender: &'a str,
    pub nonce: i64,
    pub fee: &'a str,
    pub kind: &'a str,
    pub chain_id: i64,
    pub timestamp_ms: i64,
    pub to_address: Option<&'a str>,
    pub amount: Option<&'a str>,
    pub program_id: Option<&'a str>,
    pub base_pc: Option<i64>,
    pub words_len: Option<i64>,
    pub proof_len: Option<i64>,
    pub recipients: &'a [String],
    pub asset: Option<&'a str>,
    pub bridge_amount: Option<&'a str>,
    pub to_chain: Option<i32>,
    pub bridge_to: Option<&'a str>,
    pub bridge_fee: Option<&'a str>,
    pub attestation: Option<&'a str>,
}

pub async fn insert_transaction(conn: &mut PgConnection, t: &NewTx<'_>) -> Result<()> {
    sqlx::query(
        "INSERT INTO transactions (hash, block_hash, height, tx_index, sender, nonce, fee, kind, chain_id, timestamp_ms,
                                   to_address, amount, program_id, base_pc, words_len, proof_len, recipients,
                                   asset, bridge_amount, to_chain, bridge_to, bridge_fee, attestation)
         VALUES ($1, $2, $3, $4, $5, $6, $7::numeric, $8, $9, $10, $11, $12::numeric, $13, $14, $15, $16, $17,
                 $18, $19::numeric, $20, $21, $22::numeric, $23)",
    )
    .bind(t.hash)
    .bind(t.block_hash)
    .bind(t.height)
    .bind(t.tx_index)
    .bind(t.sender)
    .bind(t.nonce)
    .bind(t.fee)
    .bind(t.kind)
    .bind(t.chain_id)
    .bind(t.timestamp_ms)
    .bind(t.to_address)
    .bind(t.amount)
    .bind(t.program_id)
    .bind(t.base_pc)
    .bind(t.words_len)
    .bind(t.proof_len)
    .bind(t.recipients)
    .bind(t.asset)
    .bind(t.bridge_amount)
    .bind(t.to_chain)
    .bind(t.bridge_to)
    .bind(t.bridge_fee)
    .bind(t.attestation)
    .execute(conn)
    .await?;
    Ok(())
}

/// The hex attestation of a `bridge_attest` transaction (`None` for other kinds).
pub async fn get_attestation(pool: &PgPool, tx_hash: &str) -> Result<Option<String>> {
    Ok(sqlx::query_scalar::<_, Option<String>>(
        "SELECT attestation FROM transactions WHERE hash = $1",
    )
    .bind(tx_hash)
    .fetch_optional(pool)
    .await?
    .flatten())
}

pub async fn insert_account_transaction(
    conn: &mut PgConnection,
    account: &str,
    tx_hash: &str,
    role: &str,
    height: i64,
    tx_index: i32,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO account_transactions (account, tx_hash, role, height, tx_index)
         VALUES ($1, $2, $3, $4, $5) ON CONFLICT DO NOTHING",
    )
    .bind(account)
    .bind(tx_hash)
    .bind(role)
    .bind(height)
    .bind(tx_index)
    .execute(conn)
    .await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn insert_receipt(
    conn: &mut PgConnection,
    tx_hash: &str,
    program: &str,
    tier: i32,
    outputs: &[i64],
    effect_to: Option<&str>,
    effect_amount: Option<&str>,
    height: i64,
    tx_index: i32,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO receipts (tx_hash, program, tier, outputs, effect_to, effect_amount, height, tx_index)
         VALUES ($1, $2, $3, $4, $5, $6::numeric, $7, $8) ON CONFLICT (tx_hash) DO NOTHING",
    )
    .bind(tx_hash)
    .bind(program)
    .bind(tier)
    .bind(outputs)
    .bind(effect_to)
    .bind(effect_amount)
    .bind(height)
    .bind(tx_index)
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn get_transaction(pool: &PgPool, hash: &str) -> Result<Option<TxRow>> {
    let sql = format!("SELECT {TX_COLS} FROM transactions WHERE hash = $1");
    Ok(sqlx::query_as::<_, TxRow>(&sql)
        .bind(hash)
        .fetch_optional(pool)
        .await?)
}

pub async fn get_receipt(pool: &PgPool, tx_hash: &str) -> Result<Option<ReceiptRow>> {
    Ok(sqlx::query_as::<_, ReceiptRow>(
        "SELECT tx_hash, program, tier, outputs, effect_to, effect_amount::text AS effect_amount, height, tx_index FROM receipts WHERE tx_hash = $1",
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

pub async fn list_transactions(
    pool: &PgPool,
    offset: i64,
    limit: i64,
    kind: Option<&str>,
    sender: Option<&str>,
    height: Option<i64>,
) -> Result<Vec<TxRow>> {
    let sql = format!(
        "SELECT {TX_COLS} FROM transactions
         WHERE ($1::text IS NULL OR kind = $1) AND ($2::text IS NULL OR sender = $2) AND ($3::bigint IS NULL OR height = $3)
         ORDER BY height DESC, tx_index DESC LIMIT $4 OFFSET $5"
    );
    Ok(sqlx::query_as::<_, TxRow>(&sql)
        .bind(kind)
        .bind(sender)
        .bind(height)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?)
}

pub async fn count_transactions(
    pool: &PgPool,
    kind: Option<&str>,
    sender: Option<&str>,
    height: Option<i64>,
) -> Result<i64> {
    Ok(sqlx::query_scalar(
        "SELECT COUNT(*) FROM transactions
         WHERE ($1::text IS NULL OR kind = $1) AND ($2::text IS NULL OR sender = $2) AND ($3::bigint IS NULL OR height = $3)",
    )
    .bind(kind)
    .bind(sender)
    .bind(height)
    .fetch_one(pool)
    .await?)
}

pub async fn list_account_transactions(
    pool: &PgPool,
    account: &str,
    offset: i64,
    limit: i64,
) -> Result<Vec<AccountTxRow>> {
    // Same columns as TX_COLS, qualified with the join alias, plus the account's role.
    let sql = format!(
        "SELECT {}, a.role
         FROM account_transactions a JOIN transactions t ON t.hash = a.tx_hash
         WHERE a.account = $1 ORDER BY a.height DESC, a.tx_index DESC, a.role ASC LIMIT $2 OFFSET $3",
        tx_cols_qualified("t")
    );
    Ok(sqlx::query_as::<_, AccountTxRow>(&sql)
        .bind(account)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?)
}

pub async fn count_account_transactions(pool: &PgPool, account: &str) -> Result<i64> {
    Ok(
        sqlx::query_scalar("SELECT COUNT(*) FROM account_transactions WHERE account = $1")
            .bind(account)
            .fetch_one(pool)
            .await?,
    )
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
