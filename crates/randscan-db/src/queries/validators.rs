//! Validator queries

use crate::{CountRow, DbError, Result, StakeHistoryRow, ValidatorRow};
use sqlx::PgPool;

/// Insert or update a validator
pub async fn upsert_validator(
    pool: &PgPool,
    validator_id: &str,
    pubkey: &str,
    stake: i64,
    commission_rate: i16,
    is_active: bool,
    timestamp: i64,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO validators (
            validator_id, pubkey, stake, commission_rate, is_active,
            first_seen, last_seen
        ) VALUES ($1, $2, $3, $4, $5, $6, $6)
        ON CONFLICT (validator_id) DO UPDATE SET
            stake = EXCLUDED.stake,
            commission_rate = EXCLUDED.commission_rate,
            is_active = EXCLUDED.is_active,
            last_seen = EXCLUDED.last_seen
        "#,
    )
    .bind(validator_id)
    .bind(pubkey)
    .bind(stake)
    .bind(commission_rate)
    .bind(is_active)
    .bind(timestamp)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get validator by ID
pub async fn get_validator_by_id(pool: &PgPool, validator_id: &str) -> Result<ValidatorRow> {
    sqlx::query_as::<_, ValidatorRow>("SELECT * FROM validators WHERE validator_id = $1")
        .bind(validator_id)
        .fetch_one(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                DbError::NotFound(format!("Validator {} not found", validator_id))
            }
            _ => e.into(),
        })
}

/// Get validators with pagination
pub async fn get_validators(
    pool: &PgPool,
    offset: i64,
    limit: i64,
    active: Option<bool>,
    sort_by: Option<&str>,
) -> Result<Vec<ValidatorRow>> {
    let order_clause = match sort_by {
        Some("stake") => "ORDER BY stake DESC",
        Some("blocks") => "ORDER BY blocks_produced DESC",
        Some("uptime") => "ORDER BY uptime_percentage DESC",
        _ => "ORDER BY stake DESC",
    };

    let query = if let Some(is_active) = active {
        format!(
            "SELECT * FROM validators WHERE is_active = {} {} LIMIT $1 OFFSET $2",
            is_active, order_clause
        )
    } else {
        format!(
            "SELECT * FROM validators {} LIMIT $1 OFFSET $2",
            order_clause
        )
    };

    sqlx::query_as::<_, ValidatorRow>(&query)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(Into::into)
}

/// Count validators
pub async fn count_validators(pool: &PgPool, active: Option<bool>) -> Result<i64> {
    let row = if let Some(is_active) = active {
        sqlx::query_as::<_, CountRow>(
            "SELECT COUNT(*) as count FROM validators WHERE is_active = $1",
        )
        .bind(is_active)
        .fetch_one(pool)
        .await?
    } else {
        sqlx::query_as::<_, CountRow>("SELECT COUNT(*) as count FROM validators")
            .fetch_one(pool)
            .await?
    };

    Ok(row.value())
}

/// Increment validator blocks produced
pub async fn increment_blocks_produced(pool: &PgPool, validator_id: &str) -> Result<()> {
    sqlx::query(
        "UPDATE validators SET blocks_produced = blocks_produced + 1 WHERE validator_id = $1",
    )
    .bind(validator_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Increment validator blocks skipped
pub async fn increment_blocks_skipped(pool: &PgPool, validator_id: &str) -> Result<()> {
    sqlx::query(
        "UPDATE validators SET blocks_skipped = blocks_skipped + 1 WHERE validator_id = $1",
    )
    .bind(validator_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Update validator vote height
pub async fn update_last_vote_height(
    pool: &PgPool,
    validator_id: &str,
    height: i64,
) -> Result<()> {
    sqlx::query("UPDATE validators SET last_vote_height = $1 WHERE validator_id = $2")
        .bind(height)
        .bind(validator_id)
        .execute(pool)
        .await?;

    Ok(())
}

/// Update validator stake
pub async fn update_validator_stake(
    pool: &PgPool,
    validator_id: &str,
    stake_delta: i64,
) -> Result<()> {
    sqlx::query("UPDATE validators SET stake = stake + $1 WHERE validator_id = $2")
        .bind(stake_delta)
        .bind(validator_id)
        .execute(pool)
        .await?;

    Ok(())
}

/// Update validator uptime
pub async fn update_validator_uptime(
    pool: &PgPool,
    validator_id: &str,
    uptime: f64,
) -> Result<()> {
    sqlx::query("UPDATE validators SET uptime_percentage = $1 WHERE validator_id = $2")
        .bind(uptime)
        .bind(validator_id)
        .execute(pool)
        .await?;

    Ok(())
}

/// Insert stake history entry
pub async fn insert_stake_history(
    pool: &PgPool,
    validator_id: &str,
    epoch: i64,
    stake: i64,
    delegators: i32,
    rewards: i64,
    timestamp: i64,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO validator_stake_history (validator_id, epoch, stake, delegators, rewards, timestamp)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (validator_id, epoch) DO UPDATE SET
            stake = EXCLUDED.stake,
            delegators = EXCLUDED.delegators,
            rewards = EXCLUDED.rewards
        "#,
    )
    .bind(validator_id)
    .bind(epoch)
    .bind(stake)
    .bind(delegators)
    .bind(rewards)
    .bind(timestamp)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get stake history for validator
pub async fn get_stake_history(
    pool: &PgPool,
    validator_id: &str,
    limit: i64,
) -> Result<Vec<StakeHistoryRow>> {
    sqlx::query_as::<_, StakeHistoryRow>(
        r#"
        SELECT * FROM validator_stake_history
        WHERE validator_id = $1
        ORDER BY epoch DESC
        LIMIT $2
        "#,
    )
    .bind(validator_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

/// Search validators by ID prefix
pub async fn search_validators(
    pool: &PgPool,
    query: &str,
    limit: i64,
) -> Result<Vec<ValidatorRow>> {
    let pattern = format!("{}%", query);
    sqlx::query_as::<_, ValidatorRow>(
        "SELECT * FROM validators WHERE validator_id LIKE $1 ORDER BY stake DESC LIMIT $2",
    )
    .bind(&pattern)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

/// Get total staked amount
pub async fn get_total_staked(pool: &PgPool) -> Result<i64> {
    #[derive(sqlx::FromRow)]
    struct SumRow {
        sum: Option<i64>,
    }

    let row = sqlx::query_as::<_, SumRow>(
        "SELECT SUM(stake) as sum FROM validators WHERE is_active = true",
    )
    .fetch_one(pool)
    .await?;

    Ok(row.sum.unwrap_or(0))
}

/// Check if validator exists
pub async fn validator_exists(pool: &PgPool, validator_id: &str) -> Result<bool> {
    let row = sqlx::query_as::<_, CountRow>(
        "SELECT COUNT(*) as count FROM validators WHERE validator_id = $1",
    )
    .bind(validator_id)
    .fetch_one(pool)
    .await?;

    Ok(row.value() > 0)
}
