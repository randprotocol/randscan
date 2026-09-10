use crate::{ProgramRow, Result};
use sqlx::{PgConnection, PgPool};

const SELECT: &str = "SELECT p.id, p.deployer, p.deploy_tx, p.deployed_at_height, p.base_pc, p.words_len, p.code_hash,
        (SELECT COUNT(*) FROM transactions t WHERE t.kind = 'call' AND t.program_id = p.id) AS call_count,
        (SELECT MAX(t.height) FROM transactions t WHERE t.kind = 'call' AND t.program_id = p.id) AS last_called_height
     FROM programs p";

#[allow(clippy::too_many_arguments)]
pub async fn insert_program(
    conn: &mut PgConnection,
    id: &str,
    deployer: &str,
    deploy_tx: &str,
    deployed_at_height: i64,
    base_pc: i64,
    words_len: i64,
    code_hash: &str,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO programs (id, deployer, deploy_tx, deployed_at_height, base_pc, words_len, code_hash)
         VALUES ($1, $2, $3, $4, $5, $6, $7) ON CONFLICT (id) DO NOTHING",
    )
    .bind(id)
    .bind(deployer)
    .bind(deploy_tx)
    .bind(deployed_at_height)
    .bind(base_pc)
    .bind(words_len)
    .bind(code_hash)
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn get_program(pool: &PgPool, id: &str) -> Result<Option<ProgramRow>> {
    let sql = format!("{SELECT} WHERE p.id = $1");
    Ok(sqlx::query_as::<_, ProgramRow>(&sql)
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

pub async fn list_programs(pool: &PgPool, offset: i64, limit: i64) -> Result<Vec<ProgramRow>> {
    let sql = format!("{SELECT} ORDER BY p.deployed_at_height DESC LIMIT $1 OFFSET $2");
    Ok(sqlx::query_as::<_, ProgramRow>(&sql)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?)
}

pub async fn count_programs(pool: &PgPool) -> Result<i64> {
    Ok(sqlx::query_scalar("SELECT COUNT(*) FROM programs")
        .fetch_one(pool)
        .await?)
}

pub async fn count_programs_by_deployer(pool: &PgPool, deployer: &str) -> Result<i64> {
    Ok(
        sqlx::query_scalar("SELECT COUNT(*) FROM programs WHERE deployer = $1")
            .bind(deployer)
            .fetch_one(pool)
            .await?,
    )
}
