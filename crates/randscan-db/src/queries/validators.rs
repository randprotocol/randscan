use crate::{Result, ValidatorRow};
use sqlx::{PgConnection, PgPool};

const SELECT: &str = "SELECT v.address, v.stake::text AS stake, v.rewards::text AS rewards, v.pending, v.payout, v.nonce, v.active, v.sort_index,
        (SELECT COUNT(*) FROM blocks b WHERE b.proposer = v.address) AS blocks_proposed,
        (SELECT MAX(b.height) FROM blocks b WHERE b.proposer = v.address) AS last_proposed_height,
        (SELECT b.timestamp_ms FROM blocks b WHERE b.proposer = v.address ORDER BY b.height DESC LIMIT 1) AS last_proposed_timestamp_ms
     FROM validators v";

/// One register entry as the indexer read it from the node.
#[derive(Debug, Clone)]
pub struct NewValidator {
    pub address: String,
    pub stake: String,
    pub rewards: String,
    /// `[{ "release_epoch": n, "amount": "units" }, ...]`
    pub pending: serde_json::Value,
    pub payout: Option<String>,
    pub nonce: i64,
    pub active: bool,
}

/// Replace the register with the node's current list (in address order, which is the
/// leader-rotation order of the active ones).
pub async fn replace_validators(conn: &mut PgConnection, set: &[NewValidator]) -> Result<()> {
    let addresses: Vec<&str> = set.iter().map(|v| v.address.as_str()).collect();
    sqlx::query("DELETE FROM validators WHERE NOT (address = ANY($1))")
        .bind(&addresses)
        .execute(&mut *conn)
        .await?;
    for (i, v) in set.iter().enumerate() {
        sqlx::query(
            "INSERT INTO validators (address, stake, rewards, pending, payout, nonce, active, sort_index)
             VALUES ($1, $2::numeric, $3::numeric, $4, $5, $6, $7, $8)
             ON CONFLICT (address) DO UPDATE SET stake = EXCLUDED.stake, rewards = EXCLUDED.rewards,
                pending = EXCLUDED.pending, payout = EXCLUDED.payout, nonce = EXCLUDED.nonce,
                active = EXCLUDED.active, sort_index = EXCLUDED.sort_index, updated_at = NOW()",
        )
        .bind(&v.address)
        .bind(&v.stake)
        .bind(&v.rewards)
        .bind(&v.pending)
        .bind(&v.payout)
        .bind(v.nonce)
        .bind(v.active)
        .bind(i as i32)
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

pub async fn list_validators(pool: &PgPool) -> Result<Vec<ValidatorRow>> {
    let sql = format!("{SELECT} ORDER BY v.sort_index ASC");
    Ok(sqlx::query_as::<_, ValidatorRow>(&sql)
        .fetch_all(pool)
        .await?)
}

pub async fn get_validator(pool: &PgPool, address: &str) -> Result<Option<ValidatorRow>> {
    let sql = format!("{SELECT} WHERE v.address = $1");
    Ok(sqlx::query_as::<_, ValidatorRow>(&sql)
        .bind(address)
        .fetch_optional(pool)
        .await?)
}

pub async fn count_validators(pool: &PgPool) -> Result<(i64, i64)> {
    Ok(sqlx::query_as::<_, (i64, i64)>(
        "SELECT COUNT(*), COUNT(*) FILTER (WHERE active) FROM validators",
    )
    .fetch_one(pool)
    .await?)
}

/// Sum of the active set's stake.
pub async fn total_stake(pool: &PgPool) -> Result<String> {
    Ok(
        sqlx::query_scalar("SELECT COALESCE(SUM(stake), 0)::text FROM validators WHERE active")
            .fetch_one(pool)
            .await?,
    )
}
