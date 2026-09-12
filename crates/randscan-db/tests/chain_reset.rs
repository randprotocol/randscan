//! Chain reset keeps user data. Runs against `DATABASE_URL`; skips when unset.

use randscan_db::{
    get_indexer_state, insert_block, insert_transaction, reset_chain_data, run_migrations,
    set_chain_id, set_next_height, upsert_account, NewBlock, NewTx, CHAIN_TABLES,
};
use sqlx::postgres::PgPoolOptions;

#[tokio::test]
async fn reset_chain_data_truncates_chain_tables_and_keeps_users() {
    let Ok(url) = std::env::var("DATABASE_URL") else {
        eprintln!("skipping: DATABASE_URL unset");
        return;
    };
    let pool = PgPoolOptions::new().max_connections(2).connect(&url).await.unwrap();
    run_migrations(&pool).await.unwrap();
    let mut conn = pool.acquire().await.unwrap();

    // One block with a bridge_burn (new columns) and a user that must survive.
    for t in CHAIN_TABLES {
        sqlx::query(&format!("TRUNCATE {t} CASCADE")).execute(&mut *conn).await.unwrap();
    }
    insert_block(
        &mut conn,
        &NewBlock {
            hash: &"ab".repeat(32),
            height: 0,
            view: 0,
            parent: &"00".repeat(32),
            proposer: "2nRd",
            timestamp_ms: 1,
            tx_root: &"00".repeat(32),
            state_root: &"00".repeat(32),
            justify_view: 0,
            tx_count: 1,
        },
    )
    .await
    .unwrap();
    insert_transaction(
        &mut conn,
        &NewTx {
            hash: &"cd".repeat(32),
            block_hash: &"ab".repeat(32),
            height: 0,
            tx_index: 0,
            sender: "2nRd",
            nonce: 0,
            fee: "1",
            kind: "bridge_burn",
            chain_id: 4,
            timestamp_ms: 1,
            to_address: None,
            amount: None,
            program_id: None,
            base_pc: None,
            words_len: None,
            proof_len: None,
            recipients: &[],
            asset: Some(&"8f".repeat(32)),
            bridge_amount: Some("99999000"),
            to_chain: Some(2),
            bridge_to: Some(&"00".repeat(32)),
            bridge_fee: Some("1000"),
            attestation: None,
        },
    )
    .await
    .unwrap();
    upsert_account(&mut conn, "2nRd", "5", 1, 0).await.unwrap();
    set_next_height(&mut conn, 1, Some(&"ab".repeat(32))).await.unwrap();
    set_chain_id(&mut conn, 4).await.unwrap();
    // The database is shared across runs and users survive the reset by design, so drop any
    // copy of this user a previous run left behind before inserting it again.
    sqlx::query("DELETE FROM users WHERE email = 'reset@example.com'").execute(&mut *conn).await.unwrap();
    sqlx::query("INSERT INTO users (email, password_hash) VALUES ('reset@example.com', 'x')")
        .execute(&mut *conn)
        .await
        .unwrap();

    let stored: (String, Option<String>, Option<i32>) = sqlx::query_as(
        "SELECT kind, bridge_amount::text, to_chain FROM transactions WHERE hash = $1",
    )
    .bind("cd".repeat(32))
    .fetch_one(&mut *conn)
    .await
    .unwrap();
    assert_eq!(stored, ("bridge_burn".into(), Some("99999000".into()), Some(2)));

    reset_chain_data(&mut conn, 5).await.unwrap();

    for t in CHAIN_TABLES {
        let n: i64 = sqlx::query_scalar(&format!("SELECT count(*) FROM {t}"))
            .fetch_one(&mut *conn)
            .await
            .unwrap();
        assert_eq!(n, 0, "{t} not emptied");
    }
    let st = get_indexer_state(&pool).await.unwrap();
    assert_eq!(st.next_height, 0);
    assert_eq!(st.last_indexed_hash, None);
    assert_eq!(st.chain_id, Some(5));
    let users: i64 = sqlx::query_scalar("SELECT count(*) FROM users WHERE email = 'reset@example.com'")
        .fetch_one(&mut *conn)
        .await
        .unwrap();
    assert_eq!(users, 1);
    let (chain_id, height): (i64, i64) =
        sqlx::query_as("SELECT chain_id, height FROM network_stats WHERE id = 1")
            .fetch_one(&mut *conn)
            .await
            .unwrap();
    assert_eq!((chain_id, height), (5, 0));
}
