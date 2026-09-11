//! RandScan against a real `shrugg-node` from `../fullnode`: a single-validator chain with the
//! faucet on, a mint and a transfer submitted over the node's own RPC, then the explorer must
//! agree with the node on every number it publishes.
//!
//! Needs `DATABASE_URL` and `SHRUGG_NODE_BIN` (path to a built `shrugg-node`, e.g.
//! `../fullnode/target/release/shrugg-node`); skips when either is unset.

mod common;

use common::*;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

const WAIT: Duration = Duration::from_secs(60);
const CHAIN_ID: u64 = 7701;

struct Node {
    child: Child,
    url: String,
    _dir: tempfile::TempDir,
}

impl Drop for Node {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn run(bin: &PathBuf, args: &[&str]) -> String {
    let out = Command::new(bin).args(args).output().expect("run shrugg-node");
    assert!(out.status.success(), "{:?} failed: {}", args, String::from_utf8_lossy(&out.stderr));
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port()
}

async fn rpc(client: &reqwest::Client, url: &str, method: &str, params: Value) -> Value {
    let body = json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params });
    let v: Value = client.post(url).json(&body).send().await.unwrap().json().await.unwrap();
    assert!(v.get("error").is_none(), "{method}: {v}");
    v["result"].clone()
}

/// keygen + genesis + init + run; returns once the node answers `shrugg_getHead`.
async fn start_node(bin: &PathBuf) -> (Node, String, String) {
    let dir = tempfile::tempdir().unwrap();
    let key = dir.path().join("validator.key.json");
    let user_key = dir.path().join("user.key.json");
    let genesis = dir.path().join("genesis.json");
    let datadir = dir.path().join("data");
    run(bin, &["keygen", "--out", key.to_str().unwrap()]);
    run(bin, &["keygen", "--out", user_key.to_str().unwrap()]);
    let validator_addr = serde_json::from_str::<Value>(&std::fs::read_to_string(&key).unwrap()).unwrap()["address"]
        .as_str()
        .unwrap()
        .to_string();
    let user_addr = serde_json::from_str::<Value>(&std::fs::read_to_string(&user_key).unwrap()).unwrap()["address"]
        .as_str()
        .unwrap()
        .to_string();
    run(
        bin,
        &[
            "genesis", "--chain-id", &CHAIN_ID.to_string(), "--validator", key.to_str().unwrap(),
            "--faucet", "--fri-profile", "test", "--out", genesis.to_str().unwrap(),
        ],
    );
    run(bin, &["init", "--datadir", datadir.to_str().unwrap(), "--genesis", genesis.to_str().unwrap()]);
    let port = free_port();
    let url = format!("http://127.0.0.1:{port}");
    let child = Command::new(bin)
        .args([
            "run", "--datadir", datadir.to_str().unwrap(), "--key", key.to_str().unwrap(),
            "--listen", "/ip4/127.0.0.1/tcp/0", "--rpc", &format!("127.0.0.1:{port}"),
            "--validator", "--no-mdns", "--block-interval-ms", "200", "--view-timeout-ms", "1000",
        ])
        .env("RUST_LOG", "warn")
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("spawn shrugg-node");
    let node = Node { child, url: url.clone(), _dir: dir };
    let client = reqwest::Client::new();
    let start = std::time::Instant::now();
    loop {
        if let Ok(r) = client
            .post(&url)
            .json(&json!({ "jsonrpc": "2.0", "id": 1, "method": "shrugg_getHead", "params": [] }))
            .send()
            .await
        {
            if r.status().is_success() {
                break;
            }
        }
        assert!(start.elapsed() < WAIT, "node did not start");
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    (node, validator_addr, user_addr)
}

#[tokio::test]
async fn explorer_agrees_with_a_real_node() {
    let Ok(bin) = std::env::var("SHRUGG_NODE_BIN") else {
        eprintln!("skipping: SHRUGG_NODE_BIN unset");
        return;
    };
    if std::env::var("DATABASE_URL").is_err() {
        eprintln!("skipping: DATABASE_URL unset");
        return;
    }
    let bin = PathBuf::from(bin);
    let client = reqwest::Client::new();
    let (node, validator_addr, user_addr) = start_node(&bin).await;

    // The node must be producing blocks on its own before we ask it to do anything.
    let start = std::time::Instant::now();
    while rpc(&client, &node.url, "shrugg_getHead", json!([])).await["height"].as_u64().unwrap() < 2 {
        assert!(start.elapsed() < WAIT, "single validator did not commit");
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    assert_eq!(rpc(&client, &node.url, "shrugg_chainId", json!([])).await, json!(CHAIN_ID));

    // Faucet mint to the user, then a transfer back to the validator (both go through consensus).
    let mint_hash = rpc(&client, &node.url, "shrugg_mint", json!([user_addr, "5000000000"])).await;
    let mint_hash = mint_hash.as_str().unwrap().to_string();
    let start = std::time::Instant::now();
    while rpc(&client, &node.url, "shrugg_getTransaction", json!([mint_hash])).await.is_null() {
        assert!(start.elapsed() < WAIT, "mint never committed");
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    let user_key = node._dir.path().join("user.key.json");
    run(
        &bin,
        &[
            "transfer", "--key", user_key.to_str().unwrap(), "--to", &validator_addr, "--amount", "1.25",
            "--rpc", &node.url,
        ],
    );

    let Some(live) = live_app(test_config(), &node.url).await else {
        return;
    };
    live.start();

    // Wait for the transfer to be committed and indexed.
    let list = live
        .wait_for(&format!("/api/v1/transactions?sender={user_addr}"), WAIT, |l| l["pagination"]["total"] == 1)
        .await;
    let transfer = &list["data"][0];
    assert_eq!(transfer["kind"], "transfer");
    assert_eq!(transfer["to"], validator_addr);
    assert_eq!(transfer["amount"], "1250000000");
    assert_eq!(transfer["chain_id"].as_u64().unwrap_or(CHAIN_ID), CHAIN_ID);

    let (status, _, mint) = call(&live.app, json_req("GET", &format!("/api/v1/transactions/{mint_hash}"), None, None)).await;
    assert_eq!(status, 200, "{mint}");
    assert_eq!(mint["kind"], "mint");
    assert_eq!(mint["to"], user_addr);
    assert_eq!(mint["amount"], "5000000000");
    assert_eq!(mint["chain_id"], CHAIN_ID);

    // Catch up to the node head, then compare what both sides publish.
    let head = rpc(&client, &node.url, "shrugg_getHead", json!([])).await["height"].as_i64().unwrap();
    live.wait_for("/api/v1/health", WAIT, |h| h["indexer"]["current_height"].as_i64().unwrap_or(-1) >= head).await;

    let node_acc = rpc(&client, &node.url, "shrugg_getAccount", json!([user_addr])).await;
    let (status, _, acc) = call(&live.app, json_req("GET", &format!("/api/v1/accounts/{user_addr}"), None, None)).await;
    assert_eq!(status, 200, "{acc}");
    assert_eq!(acc["balance"], node_acc["balance"], "explorer vs node balance");
    assert_eq!(acc["nonce"], node_acc["nonce"]);
    assert_eq!(acc["tx_count"], 2);

    let node_block = rpc(&client, &node.url, "shrugg_getBlockByHeight", json!([head])).await;
    let (_, _, block) = call(&live.app, json_req("GET", &format!("/api/v1/blocks/{head}"), None, None)).await;
    assert_eq!(block["hash"], node_block["hash"]);
    assert_eq!(block["parent"], node_block["parent"]);
    assert_eq!(block["state_root"], node_block["state_root"]);
    assert_eq!(block["proposer"], validator_addr);

    let stats = live.wait_for("/api/v1/stats", WAIT, |s| s["chain_id"] == CHAIN_ID).await;
    assert_eq!(stats["validator_count"], 1);
    assert_eq!(stats["faucet"], true);
    assert_eq!(stats["symbol"], "SHRUGG");
    let (_, _, validators) = call(&live.app, json_req("GET", "/api/v1/validators", None, None)).await;
    let list = validators.as_array().cloned().or_else(|| validators["data"].as_array().cloned()).unwrap();
    assert_eq!(list[0]["address"], validator_addr);

    let (status, _, found) = call(&live.app, json_req("GET", &format!("/api/v1/search?q={mint_hash}"), None, None)).await;
    assert_eq!(status, 200, "{found}");
    drop(node);
}
