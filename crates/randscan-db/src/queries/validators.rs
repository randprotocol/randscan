use crate::{Result, ValidatorRow};
use sqlx::{PgConnection, PgPool};

const SELECT: &str = "SELECT v.address, v.stake::text AS stake, v.sort_index,
        (SELECT COUNT(*) FROM blocks b WHERE b.proposer = v.address) AS blocks_proposed,
        (SELECT MAX(b.height) FROM blocks b WHERE b.proposer = v.address) AS last_proposed_height,
        (SELECT b.timestamp_ms FROM blocks b WHERE b.proposer = v.address ORDER BY b.height DESC LIMIT 1) AS last_proposed_timestamp_ms
     FROM validators v";

/// Replace the validator set with the node's current list (in leader-rotation order).
pub async fn replace_validators(conn: &mut PgConnection, set: &[(String, String)]) -> Result<()> {
    let addresses: Vec<String> = set.iter().map(|(a, _)| a.clone()).collect();
    sqlx::query("DELETE FROM validators WHERE NOT (address = ANY($1))")
        .bind(&addresses)
        .execute(&mut *conn)
        .await?;
    for (i, (address, stake)) in set.iter().enumerate() {
        sqlx::query(
            "INSERT INTO validators (address, stake, sort_index) VALUES ($1, $2::numeric, $3)
             ON CONFLICT (address) DO UPDATE SET stake = EXCLUDED.stake, sort_index = EXCLUDED.sort_index, updated_at = NOW()",
        )
        .bind(address)
        .bind(stake)
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

pub async fn count_validators(pool: &PgPool) -> Result<i64> {
    Ok(sqlx::query_scalar("SELECT COUNT(*) FROM validators")
        .fetch_one(pool)
        .await?)
}

pub async fn total_stake(pool: &PgPool) -> Result<String> {
    Ok(
        sqlx::query_scalar("SELECT COALESCE(SUM(stake), 0)::text FROM validators")
            .fetch_one(pool)
            .await?,
    )
}
