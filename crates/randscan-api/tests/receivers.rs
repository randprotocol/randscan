//! The receiver registry (short shielded address, fullnode spec 2026-09-17): two versions of
//! the same id are indexed and `/receivers/:address` serves the current one while
//! `/receivers/:address/history` serves both, newest first. Needs `DATABASE_URL`.

mod common;

use common::mock_node::*;
use common::*;
use randprotocol_core::notes::{word8_to_hex, KEM_EK_BYTES};
use randprotocol_core::receiver::receiver_signing_keypair;
use randprotocol_core::{ReceiverId, ReceiverRecord};
use serde_json::Value;
use std::time::Duration;

const WAIT: Duration = Duration::from_secs(20);

/// Two signed versions of one receiver id, and the id itself.
fn two_versions_of_a_record(chain_id: u64) -> (ReceiverRecord, ReceiverRecord, ReceiverId) {
    let kp = receiver_signing_keypair(&[19u8; 32]);
    let id = ReceiverId::from(kp.public_key());
    let rec1 = ReceiverRecord::sign(&kp, chain_id, 1, [1u32; 8], vec![9u8; KEM_EK_BYTES]);
    let rec2 = ReceiverRecord::sign(&kp, chain_id, 2, [2u32; 8], vec![7u8; KEM_EK_BYTES]);
    (rec1, rec2, id)
}

#[tokio::test]
async fn receivers_are_indexed_and_served_current_and_history() {
    let mut chain = MockChain::new(41, "receivers");
    let chain_id = chain.chain_id;
    let (rec1, rec2, id) = two_versions_of_a_record(chain_id);

    chain.push_block(vec![tx(
        chain_id,
        "reg-1",
        Some(bundle("reg1")),
        register_receiver(&rec1),
    )]);
    chain.push_block(vec![tx(
        chain_id,
        "reg-2",
        Some(bundle("reg2")),
        register_receiver(&rec2),
    )]);

    let node = start_mock_node(chain).await;
    let Some(live) = live_app(test_config(), &node.url).await else {
        eprintln!("skipping: DATABASE_URL unset");
        return;
    };
    {
        let mut conn = live.pool.acquire().await.unwrap();
        randscan_db::reset_chain_data(&mut conn, 0).await.unwrap();
    }
    live.start();
    live.wait_for("/api/v1/health", WAIT, |b| b["indexer"]["current_height"] == 2)
        .await;

    let (status, _, cur) = call_api(&live.app, &format!("/api/v1/receivers/{id}")).await;
    assert_eq!(status, 200, "{cur}");
    assert_eq!(cur["id"], id.to_string());
    assert_eq!(cur["version"], 2);
    assert_eq!(cur["kem_ek"], hex::encode(&rec2.kem_ek));
    assert_eq!(cur["pk"], word8_to_hex(&rec2.pk));
    assert_eq!(cur["signing_key"], rec2.signing_key.to_hex());
    assert_eq!(cur["signature"], hex::encode(rec2.signature.as_bytes()));
    assert!(cur["tx_hash"].is_string());
    assert_eq!(cur["height"], 2);

    let (status, _, hist) = call_api(&live.app, &format!("/api/v1/receivers/{id}/history")).await;
    assert_eq!(status, 200, "{hist}");
    let hist = hist.as_array().unwrap();
    assert_eq!(hist.len(), 2, "{hist:?}");
    assert_eq!(hist[0]["version"], 2, "newest first: {hist:?}");
    assert_eq!(hist[1]["version"], 1);
    assert_eq!(hist[1]["kem_ek"], hex::encode(&rec1.kem_ek));

    // A malformed address is a 400, not a lookup miss.
    let (status, _, err) = call_api(&live.app, "/api/v1/receivers/rand1nope").await;
    assert_eq!(status, 400, "{err}");

    // A well-formed address nobody ever registered is a 404.
    let (status, _, _) = call_api(
        &live.app,
        &format!("/api/v1/receivers/{}", ReceiverId([9; 32])),
    )
    .await;
    assert_eq!(status, 404);

    // Its history is an empty list, not a 404.
    let (status, _, hist) = call_api(
        &live.app,
        &format!("/api/v1/receivers/{}/history", ReceiverId([9; 32])),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(hist.as_array().unwrap().len(), 0);
}

async fn call_api(app: &axum::Router, path: &str) -> (axum::http::StatusCode, axum::http::HeaderMap, Value) {
    call(app, json_req("GET", path, None, None)).await
}
