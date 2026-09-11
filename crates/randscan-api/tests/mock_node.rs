//! Indexer, database, API and broadcast fan-out working together against a scripted node:
//! the bridge transaction kinds from the fullnode bridge merge, an unknown future kind, a block at
//! the consensus size limit (`MAX_BLOCK_TXS`, fullnode review item M1), and two hard forks under a
//! running indexer (new chain id; same chain id with a new genesis). Needs `DATABASE_URL`.

mod common;

use common::mock_node::*;
use common::*;
use randscan_core::{BroadcastEvent, TxKind};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const WAIT: Duration = Duration::from_secs(40);

#[tokio::test]
async fn indexes_bridge_kinds_and_survives_hard_forks() {
    // ----- phase 1: chain 4 with every kind the node can serve -------------------------------
    let mut chain = MockChain::new(4, "genesis-a");
    let transfer = tx(4, ALICE, 0, json!({ "type": "transfer", "to": BOB, "amount": "3500000000" }));
    chain.push_block(vec![transfer.clone()]);
    let attestation = "01000000".to_string() + &"ab".repeat(600);
    let attest = tx(4, CAROL, 0, json!({ "type": "bridge_attest", "attestation": attestation }));
    let asset = h("asset-usdc");
    let evm_to = format!("{}{}", "0".repeat(24), "f10befe1e0794722d3baf8bfd5bdac47b2a33148");
    let burn = tx(4, CAROL, 1, json!({
        "type": "bridge_burn", "asset": asset, "amount": "99999000", "to_chain": 2, "to": evm_to, "fee": "1000"
    }));
    let future = tx(4, ALICE, 1, json!({ "type": "shielded_transfer", "note": "opaque" }));
    chain.push_block(vec![attest.clone(), burn.clone(), future.clone()]);
    // M1: 2000 transactions per block is now a consensus rule; the explorer must take a full block.
    let fat: Vec<Value> = (0..2000)
        .map(|i| tx(4, ALICE, 10 + i, json!({ "type": "transfer", "to": BOB, "amount": "1" })))
        .collect();
    chain.push_block(fat);

    let node = start_mock_node(chain).await;
    let Some(live) = live_app(test_config(), &node.url).await else {
        eprintln!("skipping: DATABASE_URL unset");
        return;
    };

    // A previous run may have left this exact chain behind; start from nothing so every block
    // below is indexed (and broadcast) by this run.
    {
        let mut conn = live.pool.acquire().await.unwrap();
        randscan_db::reset_chain_data(&mut conn, 0).await.unwrap();
    }

    // Collect broadcast events continuously (the channel would drop old ones under 2000 txs).
    let events: Arc<Mutex<Vec<BroadcastEvent>>> = Arc::default();
    {
        let mut rx = live.broadcaster.subscribe();
        let events = events.clone();
        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(e) => events.lock().unwrap().push(e),
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(_) => break,
                }
            }
        });
    }
    live.start();

    live.wait_for("/api/v1/health", WAIT, |b| b["indexer"]["current_height"] == 3).await;

    let burn_hash = burn["hash"].as_str().unwrap();
    let (status, _, d) = call(&live.app, json_req("GET", &format!("/api/v1/transactions/{burn_hash}"), None, None)).await;
    assert_eq!(status, 200, "{d}");
    assert_eq!(d["kind"], "bridge_burn");
    assert_eq!(d["asset"], asset);
    assert_eq!(d["bridge_amount"], "99999000");
    assert_eq!(d["to_chain"], 2);
    assert_eq!(d["bridge_to"], evm_to);
    assert_eq!(d["bridge_fee"], "1000");
    assert!(d["to"].is_null() && d["amount"].is_null(), "foreign destination must not be a SHRUGG recipient: {d}");
    assert_eq!(d["chain_id"], 4);

    let attest_hash = attest["hash"].as_str().unwrap();
    let (_, _, d) = call(&live.app, json_req("GET", &format!("/api/v1/transactions/{attest_hash}"), None, None)).await;
    assert_eq!(d["kind"], "bridge_attest");
    assert_eq!(d["attestation"], attestation);
    // The list view never carries the (large) attestation.
    let (_, _, list) = call(&live.app, json_req("GET", "/api/v1/transactions?kind=bridge_attest", None, None)).await;
    assert_eq!(list["pagination"]["total"], 1, "{list}");
    assert!(list["data"][0].get("attestation").is_none());

    let future_hash = future["hash"].as_str().unwrap();
    let (_, _, d) = call(&live.app, json_req("GET", &format!("/api/v1/transactions/{future_hash}"), None, None)).await;
    assert_eq!(d["kind"], "other", "unknown node kinds are indexed, not dropped: {d}");
    assert_eq!(d["sender"], ALICE);
    let raw: String = sqlx::query_scalar("SELECT kind FROM transactions WHERE hash = $1")
        .bind(future_hash)
        .fetch_one(&live.pool)
        .await
        .unwrap();
    assert_eq!(raw, "shielded_transfer", "the node's own tag is kept for a later backfill");

    let (_, _, list) = call(&live.app, json_req("GET", "/api/v1/transactions?kind=bridge_burn", None, None)).await;
    assert_eq!(list["pagination"]["total"], 1);
    assert_eq!(list["data"][0]["hash"], burn_hash);
    let (status, _, _) = call(&live.app, json_req("GET", "/api/v1/transactions?kind=shielded_transfer", None, None)).await;
    assert_eq!(status, 400, "unknown kinds are not a valid filter");
    let (status, _, _) = call(&live.app, json_req("GET", "/api/v1/transactions?kind=other", None, None)).await;
    assert_eq!(status, 400);

    // The burn's foreign destination never became an account.
    let n: i64 = sqlx::query_scalar("SELECT count(*) FROM accounts WHERE address = $1")
        .bind(&evm_to)
        .fetch_one(&live.pool)
        .await
        .unwrap();
    assert_eq!(n, 0);
    // The burner did (as sender), and CAROL's history shows both bridge transactions.
    let (status, _, hist) = call(&live.app, json_req("GET", &format!("/api/v1/accounts/{CAROL}/transactions"), None, None)).await;
    assert_eq!(status, 200, "{hist}");
    assert_eq!(hist["pagination"]["total"], 2, "{hist}");

    // M1: the full-size block is served whole.
    let (status, _, b) = call(&live.app, json_req("GET", "/api/v1/blocks/3", None, None)).await;
    assert_eq!(status, 200);
    assert_eq!(b["tx_count"], 2000);
    assert_eq!(b["transactions"].as_array().unwrap().len(), 2000);
    let (_, _, list) = call(&live.app, json_req("GET", "/api/v1/transactions?height=3&limit=100&page=20", None, None)).await;
    assert_eq!(list["pagination"]["total"], 2000);
    assert_eq!(list["data"].as_array().unwrap().len(), 100);

    let stats = live.wait_for("/api/v1/stats", WAIT, |s| s["chain_id"] == 4 && s["height"] == 3).await;
    assert_eq!(stats["total_transactions"], 2004, "{stats}");
    assert_eq!(stats["validator_count"], 1);

    let (_, _, v) = call(&live.app, json_req("GET", "/api/v1/validators", None, None)).await;
    assert_eq!(v.as_array().map(|a| a.len()).or_else(|| v["data"].as_array().map(|a| a.len())), Some(1), "{v}");

    // Broadcast fan-out (what /ws subscribers receive) carried the bridge kinds.
    let got = {
        let ev = events.lock().unwrap();
        let mut kinds = Vec::new();
        for e in ev.iter() {
            if let BroadcastEvent::NewTransaction(t) = e {
                kinds.push(t.kind);
            }
        }
        kinds
    };
    assert!(got.contains(&TxKind::BridgeBurn), "broadcast kinds: {got:?}");
    assert!(got.contains(&TxKind::BridgeAttest));
    assert!(got.contains(&TxKind::Other));

    // ----- phase 2: a user and an API key exist; the fleet hard-forks to chain 5 ------------
    let body = json!({ "email": unique_email(), "password": "correct horse battery" });
    let (status, headers, _) = call(&live.app, json_req("POST", "/api/v1/auth/signup", Some(body), None)).await;
    assert_eq!(status, 201);
    let cookie = session_cookie_from(&headers);
    let (status, _, created) = call(&live.app, json_req("POST", "/api/v1/keys", Some(json!({ "name": "bot" })), Some(&cookie))).await;
    assert_eq!(status, 201, "{created}");
    let key = created["key"].as_str().unwrap().to_string();

    node.with_chain(|c| {
        *c = MockChain::new(5, "genesis-b");
        c.push_block(vec![tx(5, BOB, 0, json!({ "type": "transfer", "to": ALICE, "amount": "1" }))]);
    });

    let stats = live.wait_for("/api/v1/stats", WAIT, |s| s["chain_id"] == 5 && s["height"] == 1).await;
    assert_eq!(stats["total_transactions"], 1, "old chain data gone: {stats}");
    let (status, _, _) = call(&live.app, json_req("GET", &format!("/api/v1/transactions/{burn_hash}"), None, None)).await;
    assert_eq!(status, 404);
    let (status, _, b0) = call(&live.app, json_req("GET", "/api/v1/blocks/0", None, None)).await;
    assert_eq!(status, 200);
    assert_eq!(b0["hash"], h("block-5-genesis-b-0"));
    let stored_chain: Option<i64> = sqlx::query_scalar("SELECT chain_id FROM indexer_state WHERE id = 1")
        .fetch_one(&live.pool)
        .await
        .unwrap();
    assert_eq!(stored_chain, Some(5));

    // Users, sessions and keys survived the fork.
    let (status, _, me) = call(&live.app, json_req("GET", "/api/v1/auth/me", None, Some(&cookie))).await;
    assert_eq!(status, 200, "{me}");
    let (_, _, keys) = call(&live.app, json_req("GET", "/api/v1/keys", None, Some(&cookie))).await;
    assert_eq!(keys.as_array().unwrap().len(), 1);
    let req = axum::http::Request::builder()
        .uri("/api/v1/stats")
        .header("authorization", format!("Bearer {key}"))
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, _, _) = call(&live.app, req).await;
    assert_eq!(status, 200);

    // ----- phase 3: same chain id, re-created genesis, node head below the indexed height ---
    node.with_chain(|c| {
        c.push_block(vec![]);
        c.push_block(vec![]);
    });
    live.wait_for("/api/v1/health", WAIT, |b| b["indexer"]["current_height"] == 3).await;
    node.with_chain(|c| *c = MockChain::new(5, "genesis-c"));
    // The forced re-check is rate limited (10 s), so allow for that.
    let b0 = live
        .wait_for("/api/v1/blocks/0", WAIT, |b| b["hash"] == h("block-5-genesis-c-0"))
        .await;
    assert_eq!(b0["height"], 0);
    let (status, _, _) = call(&live.app, json_req("GET", "/api/v1/blocks/3", None, None)).await;
    assert_eq!(status, 404);
    let (status, _, _) = call(&live.app, json_req("GET", "/api/v1/auth/me", None, Some(&cookie))).await;
    assert_eq!(status, 200);
}
