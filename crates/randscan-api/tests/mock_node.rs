//! Indexer, database, API and broadcast fan-out working together against a scripted shielded
//! node: every action kind of phases S1–S3, an unknown future kind, a block at the consensus
//! transaction limit, the commitment tree and nullifier set, the S2 register/epoch/supply and
//! the S3-only register shape, and two hard forks under a running indexer (new chain id; same
//! chain id with a new genesis). Needs `DATABASE_URL`.

mod common;

use common::mock_node::*;
use common::*;
use randscan_core::{BroadcastEvent, TxKind};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const WAIT: Duration = Duration::from_secs(40);

#[tokio::test]
async fn indexes_every_shielded_kind_and_survives_hard_forks() {
    // ----- phase 1: chain 7 with every action the node can serve ---------------------------
    let mut chain = MockChain::new(7, "genesis-a");
    let transfer = tx(7, "transfer", Some(bundle("t")), json!({ "kind": "none" }));
    let mint = tx(7, "mint", None, json!({ "kind": "mint", "cm": h("cm-mint"), "amount": 100000000000u64, "minter": VALIDATOR }));
    chain.push_block(vec![transfer.clone(), mint.clone()]);

    let program = h("program-1");
    let deploy = tx(7, "deploy", Some(bundle("d")), json!({ "kind": "deploy", "program": program, "words": 412 }));
    let call_tx = tx(7, "call", Some(bundle("c")), json!({ "kind": "call", "program": program, "proof_len": 1202416, "input_envelope_len": 1280 }));
    let bond = tx(7, "bond", Some(bundle("b")), json!({ "kind": "bond", "validator": VALIDATOR_B, "amount": 1000000000000u64, "registered": true }));
    let unbond = tx(7, "unbond", None, json!({ "kind": "unbond", "validator": VALIDATOR_B, "amount": 5000000000u64, "nonce": 1 }));
    let withdraw = tx(7, "withdraw", None, json!({ "kind": "withdraw", "validator": VALIDATOR, "amount": 9000000, "nonce": 3 }));
    chain.push_block(vec![deploy.clone(), call_tx.clone(), bond.clone(), unbond.clone(), withdraw.clone()]);

    let attest = tx(7, "attest", Some(bundle("a")), json!({
        "kind": "bridge_attest", "attestation_len": 520, "recipient": SHIELDED_ADDR, "asset": 1, "asset_index": 1,
        "amount": 1000, "time": 2, "r": "aa".repeat(32), "commitment": h("deposit-attest"), "pq_signers": [0, 1]
    }));
    let rotation = tx(7, "rotation", Some(bundle("r")), json!({
        "kind": "bridge_attest", "attestation_len": 700, "recipient": SHIELDED_ADDR, "asset": null, "asset_index": null,
        "amount": null, "time": 2, "r": "00".repeat(32), "commitment": null, "pq_signers": [0, 1]
    }));
    let evm_to = format!("{}{}", "0".repeat(24), "f10befe1e0794722d3baf8bfd5bdac47b2a33148");
    let burn = tx(7, "burn", Some(bundle("bb")), json!({
        "kind": "bridge_burn", "asset": 1, "amount": 400, "relayer_fee": 100, "to_chain": 2, "token": "cc".repeat(32), "to": evm_to
    }));
    let future = tx(7, "future", Some(bundle("f")), json!({ "kind": "slash", "evidence": "opaque" }));

    // The RPL token standard (spec §4/§6) and bridge hardening's governance actions (B1/B4): a
    // token registered with an initial mint, a later mint, an authority change, a holder burn,
    // the mint pause and its lifting, and a second bridged token listed after genesis.
    chain.push_token(json!({
        "index": 3, "id": h("zusd-id"), "id_text": "rpl1zusdexampleexampleexampleexampleexampleexampleexampleeez",
        "name": "zUSD", "symbol": "zUSD", "decimals": 6,
        "authority": { "kind": "bridge", "backings": [{ "chain": 2, "token": "bb".repeat(32), "decimals": 6, "locked": "600", "minted_today": "700", "mint_day": 20345, "mint_cap_per_day": "10000000000" }] },
        "mint_nonce": 1, "total_supply": "5700", "registered_at": 3
    }));
    let zusd_register_recipient = shielded_address("zusd-register-recipient");
    let zusd_mint_recipient = shielded_address("zusd-mint-recipient");
    let register_token = tx(7, "register-zusd", Some(bundle("rt")), json!({
        "kind": "register_token", "name": "zUSD", "symbol": "zUSD", "decimals": 6, "authority": "bridge",
        "index": 3, "initial_amount": 5000,
        "initial": { "amount": 5000, "recipient": zusd_register_recipient, "time": 3, "r": "bb".repeat(32) }
    }));
    let token_mint = tx(7, "mint-zusd", Some(bundle("tm")), json!({
        "kind": "token_mint", "asset": 3, "amount": 700, "recipient": zusd_mint_recipient, "time": 3, "r": "cc".repeat(32), "nonce": 0
    }));
    let set_authority = tx(7, "set-auth-zusd", Some(bundle("sa")), json!({
        "kind": "set_authority", "asset": 3, "nonce": 1, "new_authority": VALIDATOR
    }));
    let mut token_burn_bundle = bundle("tb");
    token_burn_bundle["burn_a"] = json!(400);
    token_burn_bundle["burn_asset"] = json!(3);
    let token_burn = tx(7, "burn-zusd", Some(token_burn_bundle), json!({ "kind": "token_burn", "asset": 3, "amount": 400 }));
    let pause = tx(7, "pause", None, json!({ "kind": "pause_mints", "nonce": 4 }));
    let unpause = tx(7, "unpause", None, json!({ "kind": "unpause_mints", "nonce": 5, "pq_signers": [0, 2] }));
    let register_bridged = tx(7, "register-bridged-2", Some(bundle("rb")), json!({
        "kind": "register_bridged_token", "name": "zUSD2", "symbol": "zUSD2", "salt": "ee".repeat(4),
        "chain": 3, "token": "ff".repeat(32), "decimals": 6, "nonce": 6, "asset_id": h("zusd2-asset"), "pq_signers": [1]
    }));
    let list_backing = tx(7, "list-backing", Some(bundle("lb")), json!({
        "kind": "list_backing", "token_index": 3, "chain": 4, "token": "11".repeat(32), "decimals": 6, "nonce": 7, "pq_signers": [2]
    }));

    chain.push_block(vec![
        attest.clone(), rotation.clone(), burn.clone(), future.clone(),
        register_token.clone(), token_mint.clone(), set_authority.clone(), token_burn.clone(),
        pause.clone(), unpause.clone(), register_bridged.clone(), list_backing.clone(),
    ]);
    // 2000 transactions per block is a consensus rule; the explorer must take a full block.
    let fat: Vec<Value> = (0..2000)
        .map(|i| tx(7, &format!("fat-{i}"), Some(bundle(&format!("fat-{i}"))), json!({ "kind": "none" })))
        .collect();
    chain.push_block(fat);
    let expected_leaves = chain.leaves.len();
    chain.validators.push(MockValidator {
        address: VALIDATOR_B.into(),
        stake: "995000000000".into(),
        rewards: "0".into(),
        pending: vec![(3, "5000000000".into())],
        active: false,
    });
    chain.validators[0].rewards = "3000000".into();

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

    live.wait_for("/api/v1/health", WAIT, |b| b["indexer"]["current_height"] == 4).await;

    // A plain transfer: a bundle and nothing else.
    let transfer_hash = transfer["hash"].as_str().unwrap();
    let (status, _, d) = call_api(&live.app, &format!("/api/v1/transactions/{transfer_hash}")).await;
    assert_eq!(status, 200, "{d}");
    assert_eq!(d["kind"], "transfer");
    assert_eq!(d["has_bundle"], true);
    assert_eq!(d["fee"], "1000000");
    assert_eq!(d["bundle"]["nullifiers"], json!([h("nf1-t"), h("nf2-t"), h("nf3-t"), h("nf4-t")]));
    assert_eq!(d["bundle"]["commitments"][0], h("cm1-t"));
    assert_eq!(d["bundle"]["commitments"][3], h("cm4-t"));
    assert_eq!(d["bundle"]["proof_len"], 302857);
    assert_eq!(d["bundle"]["envelope_len"], json!([1380, 1380, 1380, 1380]));
    assert!(d["bundle"].get("asset").is_none(), "a transfer's asset must never be served: {d}");
    assert!(d["amount"].is_null() && d["validator"].is_null() && d["program"].is_null(), "{d}");
    assert!(d.get("sender").is_none() && d.get("nonce").is_none() && d.get("to").is_none(), "no account fields: {d}");
    assert_eq!(d["chain_id"], 7);

    // A mint: no bundle, a public amount, the minting validator.
    let mint_hash = mint["hash"].as_str().unwrap();
    let (_, _, d) = call_api(&live.app, &format!("/api/v1/transactions/{mint_hash}")).await;
    assert_eq!(d["kind"], "mint");
    assert_eq!(d["has_bundle"], false);
    assert!(d["bundle"].is_null());
    assert_eq!(d["fee"], "0");
    assert_eq!(d["amount"], "100000000000");
    assert_eq!(d["cm"], h("cm-mint"));
    assert_eq!(d["validator"], VALIDATOR);

    let (_, _, d) = call_api(&live.app, &format!("/api/v1/transactions/{}", deploy["hash"].as_str().unwrap())).await;
    assert_eq!(d["kind"], "deploy");
    assert_eq!(d["program"], program);
    assert_eq!(d["words_len"], 412);
    let (_, _, d) = call_api(&live.app, &format!("/api/v1/transactions/{}", call_tx["hash"].as_str().unwrap())).await;
    assert_eq!(d["kind"], "call");
    assert_eq!(d["program"], program);
    assert_eq!(d["call_proof_len"], 1202416, "a constraint-set-5 proof size is stored as reported");
    assert_eq!(d["input_envelope_len"], 1280);
    assert!(d["receipt"].is_null(), "the mock serves no receipt");

    let (_, _, d) = call_api(&live.app, &format!("/api/v1/transactions/{}", bond["hash"].as_str().unwrap())).await;
    assert_eq!(d["kind"], "bond");
    assert_eq!(d["validator"], VALIDATOR_B);
    assert_eq!(d["amount"], "1000000000000");
    assert_eq!(d["registered"], true);
    let (_, _, d) = call_api(&live.app, &format!("/api/v1/transactions/{}", unbond["hash"].as_str().unwrap())).await;
    assert_eq!(d["kind"], "unbond");
    assert_eq!(d["has_bundle"], false);
    assert_eq!(d["action_nonce"], 1);
    let (_, _, d) = call_api(&live.app, &format!("/api/v1/transactions/{}", withdraw["hash"].as_str().unwrap())).await;
    assert_eq!(d["kind"], "withdraw");
    assert_eq!(d["amount"], "9000000");
    assert_eq!(d["action_nonce"], 3);

    let (_, _, d) = call_api(&live.app, &format!("/api/v1/transactions/{}", attest["hash"].as_str().unwrap())).await;
    assert_eq!(d["kind"], "bridge_attest");
    assert_eq!(d["attestation_len"], 520);
    assert_eq!(d["recipient"], SHIELDED_ADDR);
    assert_eq!(d["asset_index"], 1);
    assert_eq!(d["amount"], "1000");
    assert_eq!(d["note_time"], 2);
    assert_eq!(d["commitment"], h("deposit-attest"));
    assert_eq!(d["pq_signers"], json!([0, 1]));
    let (_, _, d) = call_api(&live.app, &format!("/api/v1/transactions/{}", rotation["hash"].as_str().unwrap())).await;
    assert_eq!(d["kind"], "bridge_attest");
    assert!(d["asset_index"].is_null() && d["amount"].is_null(), "a rotation deposits nothing: {d}");
    assert!(d["commitment"].is_null(), "a rotation deposits nothing: {d}");

    let burn_hash = burn["hash"].as_str().unwrap();
    let (_, _, d) = call_api(&live.app, &format!("/api/v1/transactions/{burn_hash}")).await;
    assert_eq!(d["kind"], "bridge_burn");
    assert_eq!(d["asset_index"], 1);
    assert_eq!(d["amount"], "400");
    assert_eq!(d["relayer_fee"], "100");
    assert_eq!(d["to_chain"], 2);
    assert_eq!(d["bridge_to"], evm_to);
    assert_eq!(d["bridge_token"], "cc".repeat(32), "the coin being redeemed, chain 14's single-bundle burn");
    assert!(d.get("asset_bundle").is_none(), "chain 14 has one bundle per transaction, no more asset_bundle");
    assert_eq!(d["bundle"]["nullifiers"][0], h("nf1-bb"));
    assert_eq!(d["bundle"]["nullifiers"].as_array().unwrap().len(), 4);

    // The RPL token standard (spec §4/§6): registration + initial mint, a later mint, an
    // authority change, a holder burn — all public by design, none of it a "transfer".
    let (_, _, d) = call_api(&live.app, &format!("/api/v1/transactions/{}", register_token["hash"].as_str().unwrap())).await;
    assert_eq!(d["kind"], "register_token");
    assert_eq!(d["asset_index"], 3);
    assert_eq!(d["amount"], "5000", "the initial mint's amount");
    assert_eq!(d["recipient"], zusd_register_recipient);
    assert_eq!(d["token_action"]["kind"], "register_token");
    assert_eq!(d["token_action"]["name"], "zUSD");
    assert_eq!(d["token_action"]["initial"]["amount"], "5000");
    let (_, _, d) = call_api(&live.app, &format!("/api/v1/transactions/{}", token_mint["hash"].as_str().unwrap())).await;
    assert_eq!(d["kind"], "token_mint");
    assert_eq!(d["asset_index"], 3);
    assert_eq!(d["amount"], "700");
    assert_eq!(d["recipient"], zusd_mint_recipient);
    assert_eq!(d["action_nonce"], 0);
    assert_eq!(d["token_action"]["r"], "cc".repeat(32));
    let (_, _, d) = call_api(&live.app, &format!("/api/v1/transactions/{}", set_authority["hash"].as_str().unwrap())).await;
    assert_eq!(d["kind"], "set_authority");
    assert_eq!(d["asset_index"], 3);
    assert_eq!(d["action_nonce"], 1);
    assert_eq!(d["token_action"]["new_authority"], VALIDATOR);
    let (_, _, d) = call_api(&live.app, &format!("/api/v1/transactions/{}", token_burn["hash"].as_str().unwrap())).await;
    assert_eq!(d["kind"], "token_burn");
    assert_eq!(d["asset_index"], 3);
    assert_eq!(d["amount"], "400");
    assert_eq!(d["bundle"]["burn_a"], "400");
    assert_eq!(d["bundle"]["burn_asset"], 3);
    assert_eq!(d["bundle"]["burn_r"], "0");

    // Bridge hardening B1/B4: bundle-less pause/unpause, and a second bridged token + backing
    // listed after genesis by the PQ guardian quorum.
    let (_, _, d) = call_api(&live.app, &format!("/api/v1/transactions/{}", pause["hash"].as_str().unwrap())).await;
    assert_eq!(d["kind"], "pause_mints");
    assert_eq!(d["has_bundle"], false);
    assert_eq!(d["bridge_governance"]["nonce"], 4);
    let (_, _, d) = call_api(&live.app, &format!("/api/v1/transactions/{}", unpause["hash"].as_str().unwrap())).await;
    assert_eq!(d["kind"], "unpause_mints");
    assert_eq!(d["pq_signers"], json!([0, 2]));
    let (_, _, d) = call_api(&live.app, &format!("/api/v1/transactions/{}", register_bridged["hash"].as_str().unwrap())).await;
    assert_eq!(d["kind"], "register_bridged_token");
    assert_eq!(d["bridge_governance"]["name"], "zUSD2");
    assert_eq!(d["pq_signers"], json!([1]));
    let (_, _, d) = call_api(&live.app, &format!("/api/v1/transactions/{}", list_backing["hash"].as_str().unwrap())).await;
    assert_eq!(d["kind"], "list_backing");
    assert_eq!(d["asset_index"], 3);
    assert_eq!(d["bridge_governance"]["chain"], 4);

    // Privacy (task S1 item 3): a plain transfer's whole JSON never says which asset moved.
    let dump = serde_json::to_string(&call_api(&live.app, &format!("/api/v1/transactions/{transfer_hash}")).await.2).unwrap();
    assert!(!dump.contains("\"asset\""), "a transfer must never carry an asset field anywhere: {dump}");

    let future_hash = future["hash"].as_str().unwrap();
    let (_, _, d) = call_api(&live.app, &format!("/api/v1/transactions/{future_hash}")).await;
    assert_eq!(d["kind"], "other", "unknown node kinds are indexed, not dropped: {d}");
    assert_eq!(d["has_bundle"], true);
    let raw: String = sqlx::query_scalar("SELECT kind FROM transactions WHERE hash = $1")
        .bind(future_hash)
        .fetch_one(&live.pool)
        .await
        .unwrap();
    assert_eq!(raw, "slash", "the node's own tag is kept for a later backfill");

    // Filters: kind, validator, program; `sender` is gone and `none` is not a filter.
    let (_, _, list) = call_api(&live.app, "/api/v1/transactions?kind=bridge_burn").await;
    assert_eq!(list["pagination"]["total"], 1);
    assert_eq!(list["data"][0]["hash"], burn_hash);
    let (_, _, list) = call_api(&live.app, &format!("/api/v1/transactions?validator={VALIDATOR_B}")).await;
    assert_eq!(list["pagination"]["total"], 2, "{list}");
    let (_, _, list) = call_api(&live.app, &format!("/api/v1/transactions?program={program}")).await;
    assert_eq!(list["pagination"]["total"], 2, "{list}");
    let (_, _, list) = call_api(&live.app, "/api/v1/transactions?kind=transfer").await;
    assert_eq!(list["pagination"]["total"], 2001);
    for bad in ["none", "slash", "other"] {
        let (status, _, _) = call_api(&live.app, &format!("/api/v1/transactions?kind={bad}")).await;
        assert_eq!(status, 400, "{bad} is not a valid filter");
    }

    // The full-size block is served whole.
    let (status, _, b) = call_api(&live.app, "/api/v1/blocks/4").await;
    assert_eq!(status, 200);
    assert_eq!(b["tx_count"], 2000);
    assert_eq!(b["transactions"].as_array().unwrap().len(), 2000);

    // Nullifiers: all four slots of the burn's one bundle, and the lookup names the spending
    // transaction.
    let (status, _, nf) = call_api(&live.app, &format!("/api/v1/nullifiers/{}", h("nf2-bb"))).await;
    assert_eq!(status, 200, "{nf}");
    assert_eq!(nf["tx_hash"], burn_hash);
    assert_eq!(nf["height"], 3);
    let (status, _, nf4) = call_api(&live.app, &format!("/api/v1/nullifiers/{}", h("nf4-bb"))).await;
    assert_eq!(status, 200, "{nf4}");
    assert_eq!(nf4["tx_hash"], burn_hash);
    let (status, _, _) = call_api(&live.app, &format!("/api/v1/nullifiers/{}", h("never"))).await;
    assert_eq!(status, 404);

    // The commitment tree was paged in (1000 rows per page) and linked to the transactions.
    let notes = live
        .wait_for("/api/v1/notes?limit=5", WAIT, |n| n["pagination"]["total"] == expected_leaves)
        .await;
    assert_eq!(notes["data"][0]["leaf_index"], expected_leaves as i64 - 1, "newest first: {notes}");
    let (status, _, n) = call_api(&live.app, &format!("/api/v1/notes/{}", h("cm-mint"))).await;
    assert_eq!(status, 200, "{n}");
    assert_eq!(n["tx_hash"], mint_hash);
    assert_eq!(n["height"], 1);
    let (_, _, n0) = call_api(&live.app, "/api/v1/notes/0").await;
    assert_eq!(n0["height"], 0);
    assert!(n0["tx_hash"].is_null(), "a genesis deposit note has no transaction: {n0}");
    let (_, _, n) = call_api(&live.app, &format!("/api/v1/notes/{}", h("cm2-bb"))).await;
    assert_eq!(n["tx_hash"], burn_hash, "the bundle's four output slots are all leaves");

    // The chain-computed notes (task S1 item 1/3): a register_token's initial mint and a
    // token_mint each add ONE note whose envelope rides in the action, not the bundle — but it
    // is still a real tree leaf, linked back to its transaction by the indexer's own recomputed
    // commitment (there is no wire field for it, unlike a bridge_attest's).
    let register_hash = register_token["hash"].as_str().unwrap();
    let (_, _, env) = call_api(&live.app, &format!("/api/v1/transactions/{register_hash}/envelopes")).await;
    assert_eq!(env["notes"].as_array().unwrap().len(), 5, "4 bundle slots + the initial mint: {env}");
    let mint_zusd_hash = token_mint["hash"].as_str().unwrap();
    let (_, _, env) = call_api(&live.app, &format!("/api/v1/transactions/{mint_zusd_hash}/envelopes")).await;
    assert_eq!(env["notes"].as_array().unwrap().len(), 5, "4 bundle slots + the minted note: {env}");
    let derived_cm = mint_note_cm(&zusd_mint_recipient, 700, 3, 3, &"cc".repeat(32));
    assert!(
        env["notes"].as_array().unwrap().iter().any(|n| n["cm"] == derived_cm),
        "the minted note's own commitment must be among the transaction's notes: {env}"
    );

    // Envelopes: every leaf carries the node's envelope, a transaction lists the leaves it
    // created, and a call gets its sealed transcript from the node on demand.
    let (status, _, env) = call_api(&live.app, &format!("/api/v1/transactions/{mint_hash}/envelopes")).await;
    assert_eq!(status, 200, "{env}");
    assert_eq!(env["kind"], "mint");
    assert_eq!(env["notes"].as_array().unwrap().len(), 1);
    assert_eq!(env["notes"][0]["cm"], h("cm-mint"));
    assert_eq!(env["notes"][0]["envelope"]["kem_ct"], "00");
    assert!(env["call_envelope"].is_null() && env["h_in"].is_null());
    let (_, _, env) = call_api(&live.app, &format!("/api/v1/transactions/{burn_hash}/envelopes")).await;
    assert_eq!(env["notes"].as_array().unwrap().len(), 4, "the one hidden-asset bundle's four slots: {env}");
    let (_, _, env) = call_api(&live.app, &format!("/api/v1/transactions/{}/envelopes", call_tx["hash"].as_str().unwrap())).await;
    assert_eq!(env["kind"], "call");
    assert_eq!(env["call_envelope"]["body"], "0b".repeat(60));
    assert!(env["h_in"].is_null(), "no receipt indexed by the mock");
    let (_, _, page) = call_api(&live.app, "/api/v1/envelopes?from_leaf=0&limit=1000").await;
    assert_eq!(page["total_leaves"], expected_leaves);
    assert_eq!(page["notes"].as_array().unwrap().len(), 1000);
    assert_eq!(page["next_leaf"], 1000);
    let (_, _, page) = call_api(&live.app, &format!("/api/v1/envelopes?from_leaf={}&limit=1000", expected_leaves - 5)).await;
    assert_eq!(page["notes"].as_array().unwrap().len(), 5);
    assert!(page["next_leaf"].is_null());

    // Search finds a commitment and a nullifier, and no accounts.
    let (_, _, found) = call_api(&live.app, &format!("/api/v1/search?q={}", h("cm-mint"))).await;
    assert_eq!(found[0]["type"], "note", "{found}");
    let (_, _, found) = call_api(&live.app, &format!("/api/v1/search?q={}", h("nf1-t"))).await;
    assert_eq!(found[0]["type"], "nullifier", "{found}");
    let (_, _, found) = call_api(&live.app, &format!("/api/v1/search?q={VALIDATOR_B}")).await;
    assert_eq!(found[0]["type"], "validator");
    assert_eq!(found[0]["title"], "Validator (inactive)");
    let (status, _, gone) = call_api(&live.app, &format!("/api/v1/accounts/{VALIDATOR}")).await;
    assert_eq!(status, 410, "{gone}");
    assert_eq!(gone["error"], "no_accounts");
    let (status, _, _) = call_api(&live.app, &format!("/api/v1/accounts/{VALIDATOR}/transactions")).await;
    assert_eq!(status, 410);

    // Stats carry the tree, the register and the supply audit.
    let stats = live.wait_for("/api/v1/stats", WAIT, |s| s["chain_id"] == 7 && s["height"] == 4 && s["validator_count"] == 2).await;
    assert_eq!(stats["total_transactions"], 2019, "{stats}");
    assert_eq!(stats["notes"], expected_leaves);
    // 2014 bundle-carrying transactions (2000 transfers, 1 mint's bundle-less action aside, plus
    // deploy/call/bond/attest/rotation/burn/future/register_token/token_mint/set_authority/
    // token_burn/register_bridged/list_backing = 14 more), each four slots; pause_mints and
    // unpause_mints carry none.
    assert_eq!(stats["nullifiers"], 2014 * 4, "{stats}");
    assert_eq!(stats["active_validator_count"], 1);
    assert_eq!(stats["total_stake"], "100000000000000", "only the active set counts");
    assert_eq!(stats["total_supply"], "101100000000000");
    assert_eq!(stats["pool_value"], "1099997000000");
    assert_eq!(stats["epoch"], 0);
    assert_eq!(stats["epoch_blocks"], 1000);
    assert_eq!(stats["hc_bundle"], h("hc_bundle"));
    assert_eq!(stats["current_leader"], VALIDATOR);
    assert!(stats.get("total_accounts").is_none());

    let (_, _, v) = call_api(&live.app, "/api/v1/validators").await;
    let v = v.as_array().unwrap();
    assert_eq!(v.len(), 2, "{v:?}");
    assert_eq!(v[0]["address"], VALIDATOR);
    assert_eq!(v[0]["rewards"], "3000000");
    assert_eq!(v[0]["active"], true);
    assert_eq!(v[0]["share_percent"], 100.0);
    assert_eq!(v[1]["address"], VALIDATOR_B);
    assert_eq!(v[1]["active"], false);
    assert_eq!(v[1]["pending"], json!([{ "release_epoch": 3, "amount": "5000000000" }]));
    assert_eq!(v[1]["payout"], SHIELDED_ADDR);
    assert_eq!(v[1]["share_percent"], 0.0);
    let (_, _, supply) = call_api(&live.app, "/api/v1/supply").await;
    assert_eq!(supply["invariant_holds"], true);
    let (_, _, bridge) = call_api(&live.app, "/api/v1/bridge").await;
    assert_eq!(bridge["enabled"], true);
    assert_eq!(bridge["assets"][0]["index"], 1);
    // Bridge hardening B1/B3/B4 (task S1 item 4): pause state, the PQ guardian count and
    // per-backing locked/minted_today/cap all come straight through from `rand_getBridgeState`.
    assert_eq!(bridge["mint_paused"], false);
    assert_eq!(bridge["pause_nonce"], 0);
    assert_eq!(bridge["list_nonce"], 0);
    assert_eq!(bridge["registration_fee"], 1_000_000_000u64);
    assert!(bridge["pause_key"].as_str().is_some());
    assert_eq!(bridge["assets"][0]["decimals"], 6);
    assert_eq!(bridge["assets"][0]["locked"], "600");
    // Registry rows joined with the indexed flows: index 1 saw one 1000-unit deposit and one
    // 400-unit burn (the rotation carries no asset and is not counted); index 2 is Ethereum
    // USDT, registered but never used, so it is named and zero.
    let (_, _, assets) = call_api(&live.app, "/api/v1/bridge/assets").await;
    let assets = assets.as_array().expect("array");
    assert_eq!(assets.len(), 2, "{assets:?}");
    assert_eq!(assets[0]["index"], 1);
    assert_eq!(assets[0]["chain"], 2);
    assert_eq!(assets[0]["deposits"], 1);
    assert_eq!(assets[0]["deposited"], "1000");
    assert_eq!(assets[0]["burns"], 1);
    assert_eq!(assets[0]["burned"], "400");
    assert_eq!(assets[0]["outstanding"], "600");
    assert_eq!(assets[0]["symbol"], Value::Null);
    assert_eq!(assets[0]["first_height"], assets[0]["last_height"]);
    assert_eq!(assets[0]["locked"], "600", "the registry's own figure, alongside the indexed flows");
    assert_eq!(assets[1]["index"], 2);
    assert_eq!(assets[1]["symbol"], "USDT");
    assert_eq!(assets[1]["name"], "Tether USD");
    assert_eq!(assets[1]["decimals"], 6);
    assert_eq!(assets[1]["deposits"], 0);
    assert_eq!(assets[1]["outstanding"], "0");
    assert_eq!(assets[1]["last_height"], Value::Null);

    // The RPL token registry (task S1 item 2): the list page and one token's detail, its
    // backing's locked/minted_today, its deploy transaction and its public supply history.
    let (status, _, tokens_list) = call_api(&live.app, "/api/v1/tokens").await;
    assert_eq!(status, 200, "{tokens_list}");
    assert_eq!(tokens_list["enabled"], true);
    assert_eq!(tokens_list["tokens"].as_array().unwrap().len(), 1);
    assert_eq!(tokens_list["tokens"][0]["symbol"], "zUSD");
    assert_eq!(tokens_list["tokens"][0]["decimals"], 6);
    assert_eq!(tokens_list["tokens"][0]["authority"]["kind"], "bridge");
    assert_eq!(tokens_list["tokens"][0]["authority"]["backings"][0]["locked"], "600");
    assert_eq!(tokens_list["tokens"][0]["authority"]["backings"][0]["minted_today"], "700");

    for key in ["3", &h("zusd-id"), "rpl1zusdexampleexampleexampleexampleexampleexampleexampleeez"] {
        let (status, _, d) = call_api(&live.app, &format!("/api/v1/tokens/{key}")).await;
        assert_eq!(status, 200, "token lookup by {key}: {d}");
        assert_eq!(d["symbol"], "zUSD", "{key}");
        assert_eq!(d["deploy_tx"], register_hash, "the register_token transaction, by asset_index: {d}");
        let history = d["supply_history"].as_array().unwrap();
        assert_eq!(history.len(), 3, "register (initial 5000) + mint (700) + burn (400): {d}");
        assert_eq!(history[0]["kind"], "register_token");
        assert_eq!(history[0]["delta"], "5000");
        assert_eq!(history[1]["kind"], "token_mint");
        assert_eq!(history[1]["delta"], "700");
        assert_eq!(history[2]["kind"], "token_burn");
        assert_eq!(history[2]["delta"], "-400");
    }
    let (status, _, notfound) = call_api(&live.app, "/api/v1/tokens/999").await;
    assert_eq!(status, 404, "{notfound}");

    // `kind=register_token` (and every other new tag) is a valid filter, distinct from `transfer`.
    let (_, _, list) = call_api(&live.app, "/api/v1/transactions?kind=register_token").await;
    assert_eq!(list["pagination"]["total"], 1, "{list}");
    let (_, _, list) = call_api(&live.app, "/api/v1/transactions?kind=token_mint").await;
    assert_eq!(list["pagination"]["total"], 1, "{list}");
    let (_, _, tokens) = call_api(&live.app, "/api/v1/bridge/tokens").await;
    let tokens = tokens.as_array().expect("array");
    assert_eq!(tokens.len(), 8);
    assert_eq!(tokens[0], json!({
        "symbol": "USDT", "name": "Tether USD", "chain": 2, "chain_name": "Ethereum", "standard": "ERC-20",
        "address": "0xdAC17F958D2ee523a2206206994597C13D831ec7",
        "token": format!("{}dac17f958d2ee523a2206206994597c13d831ec7", "0".repeat(24)),
        "decimals": 6, "status": "allowed",
        "explorer_url": "https://etherscan.io/token/0xdac17f958d2ee523a2206206994597c13d831ec7"
    }));
    assert_eq!(tokens.iter().filter(|t| t["status"] == "discontinued").count(), 1);
    let (_, _, p) = call_api(&live.app, &format!("/api/v1/programs/{program}")).await;
    assert_eq!(p["call_count"], 1);
    assert!(p.get("deployer").is_none(), "{p}");

    // Broadcast fan-out (what /ws subscribers receive) carried every kind.
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
    for k in [
        TxKind::Transfer, TxKind::Mint, TxKind::Deploy, TxKind::Call, TxKind::Bond, TxKind::Unbond,
        TxKind::Withdraw, TxKind::BridgeAttest, TxKind::BridgeBurn, TxKind::RegisterToken, TxKind::TokenMint,
        TxKind::SetAuthority, TxKind::TokenBurn, TxKind::PauseMints, TxKind::UnpauseMints,
        TxKind::RegisterBridgedToken, TxKind::ListBacking, TxKind::Other,
    ] {
        assert!(got.contains(&k), "broadcast kinds miss {k}: {got:?}");
    }

    // ----- phase 2: a user and an API key exist; the fleet hard-forks to chain 8 (an S3-only
    // node: register without pending/active, no epoch, no supply) ----------------------------
    let body = json!({ "email": unique_email(), "password": "correct horse battery" });
    let (status, headers, _) = call(&live.app, json_req("POST", "/api/v1/auth/signup", Some(body), None)).await;
    assert_eq!(status, 201);
    let cookie = session_cookie_from(&headers);
    let (status, _, created) = call(&live.app, json_req("POST", "/api/v1/keys", Some(json!({ "name": "bot" })), Some(&cookie))).await;
    assert_eq!(status, 201, "{created}");
    let key = created["key"].as_str().unwrap().to_string();

    node.with_chain(|c| {
        *c = MockChain::new(8, "genesis-b");
        c.pre_s2 = true;
        c.push_block(vec![tx(8, "t", Some(bundle("t8")), json!({ "kind": "none" }))]);
    });

    let stats = live.wait_for("/api/v1/stats", WAIT, |s| s["chain_id"] == 8 && s["height"] == 1).await;
    assert_eq!(stats["total_transactions"], 1, "old chain data gone: {stats}");
    assert_eq!(stats["notes"], 5, "the genesis note plus the one bundle's four output slots");
    assert!(stats["epoch"].is_null() && stats["pool_value"].is_null(), "S2 fields absent on an S3 node: {stats}");
    assert_eq!(stats["total_supply"], "0");
    assert_eq!(stats["active_validator_count"], 1, "every register entry is active before S2");
    let (status, _, _) = call_api(&live.app, &format!("/api/v1/transactions/{burn_hash}")).await;
    assert_eq!(status, 404);
    let (status, _, _) = call_api(&live.app, &format!("/api/v1/notes/{}", h("cm-mint"))).await;
    assert_eq!(status, 404);
    let (status, _, _) = call_api(&live.app, "/api/v1/supply").await;
    assert_eq!(status, 404);
    let (status, _, b0) = call_api(&live.app, "/api/v1/blocks/0").await;
    assert_eq!(status, 200);
    assert_eq!(b0["hash"], h("block-8-genesis-b-0"));
    let (stored_chain, next_leaf): (Option<i64>, i64) =
        sqlx::query_as("SELECT chain_id, next_leaf FROM indexer_state WHERE id = 1")
            .fetch_one(&live.pool)
            .await
            .unwrap();
    assert_eq!(stored_chain, Some(8));
    assert_eq!(next_leaf, 5);

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
    node.with_chain(|c| *c = MockChain::new(8, "genesis-c"));
    // The forced re-check is rate limited (10 s), so allow for that.
    let b0 = live
        .wait_for("/api/v1/blocks/0", WAIT, |b| b["hash"] == h("block-8-genesis-c-0"))
        .await;
    assert_eq!(b0["height"], 0);
    let (status, _, _) = call_api(&live.app, "/api/v1/blocks/3").await;
    assert_eq!(status, 404);
    let notes = live.wait_for("/api/v1/notes", WAIT, |n| n["pagination"]["total"] == 1).await;
    assert_eq!(notes["data"][0]["cm"], h("genesis-note-8-genesis-c"), "the tree was re-read from leaf 0");
    let (status, _, _) = call(&live.app, json_req("GET", "/api/v1/auth/me", None, Some(&cookie))).await;
    assert_eq!(status, 200);
}

async fn call_api(app: &axum::Router, path: &str) -> (axum::http::StatusCode, axum::http::HeaderMap, Value) {
    call(app, json_req("GET", path, None, None)).await
}
