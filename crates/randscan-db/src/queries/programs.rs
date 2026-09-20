use crate::{ProgramRow, Result};
use sqlx::{PgConnection, PgPool};

const SELECT: &str = "SELECT p.id, p.deploy_tx, p.deployed_at_height, p.base_pc, p.words_len, p.code_hash,
        p.public_words_len, p.public_digest,
        (SELECT COUNT(*) FROM transactions t WHERE t.kind = 'call' AND t.program_id = p.id) AS call_count,
        (SELECT MAX(t.height) FROM transactions t WHERE t.kind = 'call' AND t.program_id = p.id) AS last_called_height
     FROM programs p";

pub struct NewProgram<'a> {
    pub id: &'a str,
    pub deploy_tx: &'a str,
    pub deployed_at_height: i64,
    pub base_pc: i64,
    pub words_len: i64,
    pub code_hash: &'a str,
    /// The deploy-time public input: its length (0 without one) and its digest.
    pub public_words_len: i64,
    pub public_digest: Option<&'a str>,
}

pub async fn insert_program(conn: &mut PgConnection, p: &NewProgram<'_>) -> Result<()> {
    sqlx::query(
        "INSERT INTO programs (id, deploy_tx, deployed_at_height, base_pc, words_len, code_hash,
                               public_words_len, public_digest)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) ON CONFLICT (id) DO NOTHING",
    )
    .bind(p.id)
    .bind(p.deploy_tx)
    .bind(p.deployed_at_height)
    .bind(p.base_pc)
    .bind(p.words_len)
    .bind(p.code_hash)
    .bind(p.public_words_len)
    .bind(p.public_digest)
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
