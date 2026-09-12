//! RandScan against a real shielded `shrugg-node` from `../fullnode` (branch `shielded-s3` or
//! later): a single-validator chain with the faucet on, a faucet mint and a shielded transfer
//! submitted through the wallet, then the explorer must agree with the node on every public
//! number: the bundle's nullifiers and commitments, the mint's note, the tree, the register.
//!
//! Needs `DATABASE_URL` and `SHRUGG_NODE_BIN` (path to a built `shrugg-node`); skips when either
//! is unset. The wallet CLI (`shrugg`) must sit next to the node binary or be named by
//! `SHRUGG_CLI`; it proves the bundle locally under the chain's `test` FRI profile. With the
//! wallet the test also deploys a guest, proves and submits one confidential call, and checks
//! the explorer's receipt (tier, outputs, `h_in`) against the node's — the proof is made under
//! whatever zkVM constraint set the node was built with, so this is what notices a zkVM
//! re-sync changing the receipt shape.

mod common;

use common::*;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

const WAIT: Duration = Duration::from_secs(90);
const CHAIN_ID: u64 = 7701;

struct Node {
    child: Child,
    url: String,
    dir: tempfile::TempDir,
}

impl Drop for Node {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn run(bin: &PathBuf, args: &[&str]) -> String {
    let out = Command::new(bin).args(args).output().expect("run binary");
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

/// The hash after `submitted <what> ` in the wallet's output.
fn submitted_hash(out: &str, what: &str) -> String {
    out.lines()
        .find_map(|l| l.strip_prefix(&format!("submitted {what} ")))
        .and_then(|r| r.split(' ').next())
        .unwrap_or_else(|| panic!("no `submitted {what}` line in: {out}"))
        .to_string()
}

/// keygen + genesis + init + run; returns once the node answers `shrugg_getHead`.
async fn start_node(bin: &PathBuf, cli: &PathBuf) -> (Node, String, String) {
    let dir = tempfile::tempdir().unwrap();
    let key = dir.path().join("validator.key.json");
    let wallet = dir.path().join("wallet.key.json");
    let genesis = dir.path().join("genesis.json");
    let datadir = dir.path().join("data");
    run(bin, &["keygen", "--out", key.to_str().unwrap()]);
    let validator_addr = serde_json::from_str::<Value>(&std::fs::read_to_string(&key).unwrap()).unwrap()["address"]
        .as_str()
        .unwrap()
        .to_string();
    run(cli, &["keygen", "--key", wallet.to_str().unwrap()]);
    let wallet_addr = run(cli, &["address", "--key", wallet.to_str().unwrap()]).trim().to_string();
    assert!(wallet_addr.starts_with("shrugg1"), "not a shielded address: {wallet_addr}");
    run(
        bin,
        &[
            "genesis", "--chain-id", &CHAIN_ID.to_string(), "--validator", key.to_str().unwrap(),
            "--faucet", "--fri-profile", "test", "--alloc", &format!("{wallet_addr}=1000"),
            "--out", genesis.to_str().unwrap(),
        ],
    );
    run(bin, &["init", "--datadir", datadir.to_str().unwrap(), "--genesis", genesis.to_str().unwrap()]);
    let port = free_port();
    let url = format!("http://127.0.0.1:{port}");
    // 500 ms blocks: a bundle's anchor is valid for 256 blocks, so the wallet has about two
    // minutes to prove under the test profile before an honest transfer would be refused.
    let child = Command::new(bin)
        .args([
            "run", "--datadir", datadir.to_str().unwrap(), "--key", key.to_str().unwrap(),
            "--listen", "/ip4/127.0.0.1/tcp/0", "--rpc", &format!("127.0.0.1:{port}"),
            "--validator", "--no-mdns", "--block-interval-ms", "500", "--view-timeout-ms", "2000",
        ])
        .env("RUST_LOG", "warn")
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("spawn shrugg-node");
    let node = Node { child, url: url.clone(), dir };
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
    (node, validator_addr, wallet_addr)
}

#[tokio::test]
async fn explorer_agrees_with_a_real_shielded_node() {
    let Ok(bin) = std::env::var("SHRUGG_NODE_BIN") else {
        eprintln!("skipping: SHRUGG_NODE_BIN unset");
        return;
    };
    if std::env::var("DATABASE_URL").is_err() {
        eprintln!("skipping: DATABASE_URL unset");
        return;
    }
    let bin = PathBuf::from(bin);
    let cli = std::env::var("SHRUGG_CLI").map(PathBuf::from).unwrap_or_else(|_| bin.with_file_name("shrugg"));
    assert!(cli.is_file(), "no wallet CLI at {}: set SHRUGG_CLI", cli.display());
    let client = reqwest::Client::new();
    let (node, validator_addr, _wallet_addr) = start_node(&bin, &cli).await;
    let wallet = node.dir.path().join("wallet.key.json");
    let wallet = wallet.to_str().unwrap();

    // The node must be producing blocks on its own before we ask it to do anything.
    let start = std::time::Instant::now();
    while rpc(&client, &node.url, "shrugg_getHead", json!([])).await["height"].as_u64().unwrap() < 2 {
        assert!(start.elapsed() < WAIT, "single validator did not commit");
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    assert_eq!(rpc(&client, &node.url, "shrugg_chainId", json!([])).await, json!(CHAIN_ID));

    // Faucet mint into the wallet (validator-signed, no bundle), then a shielded transfer from
    // the wallet to a second wallet (a proved 2-in-2-out bundle).
    let out = run(&cli, &["faucet", "--key", wallet, "--rpc", &node.url, "--amount", "50"]);
    let mint_hash = submitted_hash(&out, "mint");
    let wallet2 = node.dir.path().join("wallet2.key.json");
    run(&cli, &["keygen", "--key", wallet2.to_str().unwrap()]);
    let wallet2_addr = run(&cli, &["address", "--key", wallet2.to_str().unwrap()]).trim().to_string();
    let out = run(&cli, &["send", &wallet2_addr, "1.25", "--key", wallet, "--rpc", &node.url]);
    let transfer_hash = submitted_hash(&out, "transfer");

    let Some(live) = live_app(test_config(), &node.url).await else {
        return;
    };
    live.start();

    // The transfer: the explorer shows exactly the bundle's public fields, nothing more.
    let node_transfer = rpc(&client, &node.url, "shrugg_getTransaction", json!([transfer_hash])).await;
    assert!(node_transfer.is_object(), "transfer not committed: {node_transfer}");
    let transfer = live
        .wait_for(&format!("/api/v1/transactions/{transfer_hash}"), WAIT, |t| t["kind"] == "transfer")
        .await;
    let nb = &node_transfer["tx"]["bundle"];
    assert_eq!(transfer["has_bundle"], true);
    assert_eq!(transfer["bundle"]["anchor"], nb["anchor"]);
    assert_eq!(transfer["bundle"]["nullifiers"], nb["nullifiers"]);
    assert_eq!(transfer["bundle"]["commitments"], nb["commitments"]);
    assert_eq!(transfer["bundle"]["fee"], nb["fee"].as_u64().unwrap().to_string());
    assert_eq!(transfer["fee"], nb["fee"].as_u64().unwrap().to_string());
    assert_eq!(transfer["bundle"]["proof_len"], nb["proof_len"]);
    assert_eq!(transfer["bundle"]["time"], nb["time"]);
    assert_eq!(transfer["height"], node_transfer["height"]);
    assert!(transfer["amount"].is_null() && transfer.get("sender").is_none(), "{transfer}");
    assert_eq!(transfer["chain_id"], CHAIN_ID);

    // The mint: no bundle, a public amount and commitment.
    let node_mint = rpc(&client, &node.url, "shrugg_getTransaction", json!([mint_hash])).await;
    let (status, _, mint) = call(&live.app, json_req("GET", &format!("/api/v1/transactions/{mint_hash}"), None, None)).await;
    assert_eq!(status, 200, "{mint}");
    assert_eq!(mint["kind"], "mint");
    assert_eq!(mint["has_bundle"], false);
    assert_eq!(mint["amount"], "50000000000");
    assert_eq!(mint["cm"], node_mint["tx"]["action"]["cm"]);
    assert_eq!(mint["validator"], validator_addr);

    // Catch up to the node head, then compare what both sides publish.
    let head = rpc(&client, &node.url, "shrugg_getHead", json!([])).await["height"].as_i64().unwrap();
    live.wait_for("/api/v1/health", WAIT, |h| h["indexer"]["current_height"].as_i64().unwrap_or(-1) >= head).await;

    let node_block = rpc(&client, &node.url, "shrugg_getBlockByHeight", json!([head])).await;
    let (_, _, block) = call(&live.app, json_req("GET", &format!("/api/v1/blocks/{head}"), None, None)).await;
    assert_eq!(block["hash"], node_block["hash"]);
    assert_eq!(block["parent"], node_block["parent"]);
    assert_eq!(block["state_root"], node_block["state_root"]);
    assert_eq!(block["proposer"], validator_addr);

    // The tree: one genesis note, one mint note, two transfer outputs; every leaf served.
    let tree = rpc(&client, &node.url, "shrugg_getTreeInfo", json!([])).await;
    assert_eq!(tree["next_index"], 4, "{tree}");
    let notes = live.wait_for("/api/v1/notes", WAIT, |n| n["pagination"]["total"] == 4).await;
    let cm_out: Vec<&str> = nb["commitments"].as_array().unwrap().iter().map(|c| c.as_str().unwrap()).collect();
    for cm in &cm_out {
        let (status, _, n) = call(&live.app, json_req("GET", &format!("/api/v1/notes/{cm}"), None, None)).await;
        assert_eq!(status, 200, "{n}");
        assert_eq!(n["tx_hash"], transfer_hash);
    }
    let (_, _, genesis_note) = call(&live.app, json_req("GET", "/api/v1/notes/0", None, None)).await;
    assert!(genesis_note["tx_hash"].is_null(), "{genesis_note}");
    assert_eq!(notes["data"].as_array().unwrap().len(), 4);
    for nf in nb["nullifiers"].as_array().unwrap() {
        let (status, _, n) = call(&live.app, json_req("GET", &format!("/api/v1/nullifiers/{}", nf.as_str().unwrap()), None, None)).await;
        assert_eq!(status, 200, "{n}");
        assert_eq!(n["tx_hash"], transfer_hash);
    }

    let node_status = rpc(&client, &node.url, "shrugg_status", json!([])).await;
    let stats = live.wait_for("/api/v1/stats", WAIT, |s| s["chain_id"] == CHAIN_ID && s["notes"] == 4).await;
    assert_eq!(stats["nullifiers"], node_status["nullifiers"]);
    assert_eq!(stats["hc_bundle"], node_status["hc_bundle"]);
    assert_eq!(stats["validator_count"], 1);
    assert_eq!(stats["faucet"], true);
    assert_eq!(stats["symbol"], "SHRUGG");
    let node_validators = rpc(&client, &node.url, "shrugg_getValidators", json!([])).await;
    let (_, _, validators) = call(&live.app, json_req("GET", "/api/v1/validators", None, None)).await;
    let list = validators.as_array().unwrap();
    assert_eq!(list[0]["address"], validator_addr);
    assert_eq!(list[0]["stake"], node_validators[0]["stake"]);
    assert_eq!(list[0]["active"], true);
    // The proposer earned the transfer's fee (and the deploy/call fees below, later).
    let rewards: u128 = list[0]["rewards"].as_str().unwrap().parse().unwrap();
    assert!(rewards >= nb["fee"].as_u64().unwrap() as u128, "{list:?}");

    let (status, _, found) = call(&live.app, json_req("GET", &format!("/api/v1/search?q={}", cm_out[0]), None, None)).await;
    assert_eq!(status, 200, "{found}");
    assert_eq!(found[0]["type"], "note");
    let (status, _, gone) = call(&live.app, json_req("GET", &format!("/api/v1/accounts/{validator_addr}"), None, None)).await;
    assert_eq!(status, 410, "{gone}");

    // Confidential call: deploy the private_payment guest, prove locally (test FRI profile),
    // submit with its input transcript, and compare the explorer's view with the node's.
    let program_json = node.dir.path().join("program.json");
    run(&cli, &["program", "build", "--guest", "private_payment", "--arg", "100", "--out", program_json.to_str().unwrap()]);
    let out = run(&cli, &["program", "deploy", program_json.to_str().unwrap(), "--rpc", &node.url, "--key", wallet]);
    let deploy_hash = submitted_hash(&out, "deploy");
    let program_id = out
        .lines()
        .find_map(|l| l.strip_prefix("program id: "))
        .and_then(|r| r.split(' ').next())
        .unwrap_or_else(|| panic!("no program id in deploy output: {out}"))
        .to_string();
    let out = run(
        &cli,
        &["call", &program_id, "--input", "400", "--input", "250", "--input", "0", "--input", "0", "--rpc", &node.url, "--key", wallet],
    );
    let call_hash = submitted_hash(&out, "call");

    let node_receipt = rpc(&client, &node.url, "shrugg_getReceipt", json!([call_hash])).await;
    assert!(node_receipt.is_object(), "node has no receipt for {call_hash}: {node_receipt}");
    let tx = live.wait_for(&format!("/api/v1/transactions/{call_hash}"), WAIT, |t| t["receipt"].is_object()).await;
    assert_eq!(tx["kind"], "call");
    assert_eq!(tx["program"], program_id);
    assert_eq!(tx["has_bundle"], true);
    assert!(tx["call_proof_len"].as_u64().unwrap_or(0) > 0, "{tx}");
    assert!(tx["input_envelope_len"].as_u64().unwrap_or(0) > 0, "the wallet publishes a transcript by default: {tx}");
    let receipt = &tx["receipt"];
    assert_eq!(receipt["tier"], node_receipt["tier"]);
    assert_eq!(receipt["outputs"], node_receipt["outputs"]);
    assert_eq!(receipt["h_in"], node_receipt["h_in"]);
    assert_eq!(receipt["height"], node_receipt["height"]);
    assert!(receipt.get("effect").is_none(), "effect kind 1 is gone with the accounts: {receipt}");

    let (_, _, deploy) = call(&live.app, json_req("GET", &format!("/api/v1/transactions/{deploy_hash}"), None, None)).await;
    assert_eq!(deploy["kind"], "deploy");
    assert_eq!(deploy["program"], program_id);
    let node_program = rpc(&client, &node.url, "shrugg_getProgram", json!([program_id])).await;
    let program = live.wait_for(&format!("/api/v1/programs/{program_id}"), WAIT, |p| p["call_count"] == 1).await;
    assert_eq!(program["code_hash"], node_program["code_hash"]);
    assert_eq!(program["words_len"], node_program["words_len"]);
    assert_eq!(program["deploy_tx"], deploy_hash);
    assert!(program.get("deployer").is_none(), "{program}");

    // Every leaf the node holds is indexed, including the two bundles that paid for the
    // deploy and the call (self-transfers of zero, four more commitments).
    let head = rpc(&client, &node.url, "shrugg_getHead", json!([])).await["height"].as_i64().unwrap();
    live.wait_for("/api/v1/health", WAIT, |h| h["indexer"]["current_height"].as_i64().unwrap_or(-1) >= head).await;
    let tree = rpc(&client, &node.url, "shrugg_getTreeInfo", json!([])).await;
    let stats = live.wait_for("/api/v1/stats", WAIT, |s| s["notes"] == tree["next_index"]).await;
    assert_eq!(stats["nullifiers"], tree["nullifiers"]);
    let (_, _, notes) = call(&live.app, json_req("GET", "/api/v1/notes?limit=1", None, None)).await;
    assert_eq!(notes["pagination"]["total"], tree["next_index"]);
    drop(node);
}
