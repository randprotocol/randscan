use crate::amount;
use serde::{Deserialize, Serialize};

/// A row of the bridge's asset registry (`rand_getAssets`, and `rand_getBridgeState.assets`).
/// `index` is the `asset` word a note of that asset carries; index 0 is RAND and never appears
/// here. `decimals`/`locked`/`minted_today`/`mint_day` are bridge hardening B1 — one row per
/// *backing* (a token with several source coins, spec §12, lists once per backing, all sharing
/// `index`); `None` on a node predating B1.
///
/// The node's own `asset_json` renders `locked`/`minted_today`/`mint_cap_per_day` as JSON
/// **numbers** here (pinned by its test: `"locked": 600, "mint_cap_per_day": 100_000u64 *
/// 100_000_000, "minted_today": 1_000`) — a different encoding from the *same* fields on
/// `TokenBacking` (the token RPC's `backing_json`, which sends `locked`/`minted_today` as
/// strings). `amount::amount_opt` accepts either, so this struct does not have to track which
/// renderer sent which.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BridgeAsset {
    pub index: i64,
    pub chain: i64,
    pub token: String,
    pub asset_id: String,
    #[serde(default)]
    pub decimals: Option<i32>,
    #[serde(default, deserialize_with = "amount::amount_opt")]
    pub locked: Option<String>,
    #[serde(default, deserialize_with = "amount::amount_opt")]
    pub minted_today: Option<String>,
    #[serde(default)]
    pub mint_day: Option<i64>,
    /// The daily mint cap for this backing (bridge hardening B1).
    #[serde(default, deserialize_with = "amount::amount_opt")]
    pub mint_cap_per_day: Option<String>,
}

/// The bridge's public state (`rand_getBridgeState`). Bridged value is notes, so there are no
/// balances here; a chain without a bridge section reports `enabled: false` and nothing else.
///
/// `mint_paused`/`pause_nonce`/`list_nonce`/`pause_key`/`pq_guardians`/`registration_fee` are
/// bridge hardening B1/B3/B4 — `None`/empty on a node predating them.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeState {
    pub enabled: bool,
    #[serde(default)]
    pub emitter: Option<String>,
    /// source chain id -> the emitter address trusted there
    #[serde(default)]
    pub emitters: std::collections::BTreeMap<String, String>,
    #[serde(default)]
    pub guardian_set_index: Option<i64>,
    #[serde(default)]
    pub guardians: Vec<String>,
    /// The genesis PQ (Dilithium2) guardian set, hex, index-aligned with `guardians`. Never moves
    /// on a rotation.
    #[serde(default)]
    pub pq_guardians: Vec<String>,
    /// B1: while true, every transfer attest is refused (burns and rotations stay open).
    #[serde(default)]
    pub mint_paused: bool,
    /// B1: what the next `pause_mints` / `unpause_mints` must carry.
    #[serde(default)]
    pub pause_nonce: Option<i64>,
    /// B4: what the next `list_backing` / `register_bridged_token` must carry.
    #[serde(default)]
    pub list_nonce: Option<i64>,
    /// B1: the one Dilithium2 key that may pause minting (it can never unpause), hex.
    #[serde(default)]
    pub pause_key: Option<String>,
    /// B4: what a `register_bridged_token` owes past the bundle base. A number (not a decimal
    /// string, unlike every amount above): `docs/rpc.md`'s own example renders it as one, and it
    /// is a chain-wide constant, not a value that grows with usage the way a supply does. Parsed
    /// tolerantly (`amount::int_opt`) in case a future node renders it as a string instead, the
    /// way `TokenBacking.locked` and this same struct's `locked` disagree today.
    #[serde(default, deserialize_with = "amount::int_opt")]
    pub registration_fee: Option<i64>,
    #[serde(default)]
    pub burn_sequence: Option<i64>,
    /// Gone from `rand_getBridgeState` since the bridge's own registry no longer predicts an
    /// index (a bridged token is listed, via the token registry, before it can be deposited);
    /// kept here, always `None`, for a node old enough to still send it.
    #[serde(default)]
    pub next_index: Option<i64>,
    #[serde(default)]
    pub assets: Vec<BridgeAsset>,
}

/// One registered bridged asset with everything the chain has seen of it: the registry row plus
/// the deposits that minted it and the burns that sent it back. Amounts are decimal strings of
/// the asset's own smallest unit (the source chain's token decimals); `deposited - burned` is the
/// supply of this asset held in shielded notes right now.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BridgeAssetActivity {
    pub index: i64,
    /// bridge chain id of the token's home (2 Ethereum, 3 BSC, 4 Tron, 5 Solana)
    pub chain: i64,
    /// the token's address on its home chain as the registry stores it (32 bytes hex)
    pub token: String,
    pub asset_id: String,
    /// set when `(chain, token)` is on the explorer's approved list (see `APPROVED_TOKENS`)
    pub symbol: Option<String>,
    pub name: Option<String>,
    /// the token's decimals on its home chain; amounts below are still in bridge units (8)
    pub decimals: Option<u8>,
    pub deposits: i64,
    pub deposited: String,
    pub burns: i64,
    pub burned: String,
    /// `deposited - burned`
    pub outstanding: String,
    pub first_height: Option<i64>,
    pub last_height: Option<i64>,
    /// This backing's own locked amount, straight from the registry (bridge hardening B1);
    /// `outstanding` above is the indexer's own reconciliation from indexed flows — the two
    /// should agree, and a difference is worth an operator's attention.
    pub locked: Option<String>,
    pub minted_today: Option<String>,
    pub mint_cap_per_day: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The exact JSON the node's own pinned test emits
    /// (`bridge_state_reports_guardians_emitters_and_the_registry`,
    /// `crates/randprotocol-node/src/rpc.rs`): `locked`/`minted_today`/`mint_cap_per_day` as bare
    /// JSON numbers. Before the tolerant `amount::amount_opt` deserializer this failed closed —
    /// `rand_getBridgeState`/`rand_getAssets` never parsed on any chain-14 node with a bridged
    /// token, so the bridge cache stayed empty forever.
    #[test]
    fn an_asset_row_with_numeric_amounts_parses_like_the_nodes_own_pinned_fixture() {
        let v = serde_json::json!({
            "index": 1, "chain": 2, "token": "aa".repeat(32), "asset_id": "bb".repeat(32),
            "decimals": 8, "locked": 600,
            "mint_cap_per_day": 100_000u64 * 100_000_000, "minted_today": 1_000, "mint_day": 0,
        });
        let asset: BridgeAsset = serde_json::from_value(v).expect("the node's own numeric encoding must parse");
        assert_eq!(asset.locked.as_deref(), Some("600"));
        assert_eq!(asset.minted_today.as_deref(), Some("1000"));
        assert_eq!(asset.mint_cap_per_day.as_deref(), Some("10000000000000"));
        assert_eq!(asset.mint_day, Some(0));
    }

    /// A hypothetical node that instead sends these as decimal strings (the token RPC's
    /// `backing_json` convention for `locked`/`minted_today`) must parse identically.
    #[test]
    fn an_asset_row_with_string_amounts_also_parses() {
        let v = serde_json::json!({
            "index": 1, "chain": 2, "token": "aa".repeat(32), "asset_id": "bb".repeat(32),
            "decimals": 8, "locked": "600", "mint_cap_per_day": "10000000000000", "minted_today": "1000", "mint_day": 0,
        });
        let asset: BridgeAsset = serde_json::from_value(v).unwrap();
        assert_eq!(asset.locked.as_deref(), Some("600"));
        assert_eq!(asset.minted_today.as_deref(), Some("1000"));
        assert_eq!(asset.mint_cap_per_day.as_deref(), Some("10000000000000"));
    }

    /// The whole `rand_getBridgeState` reply, numeric amounts and `registration_fee` a bare
    /// number, no `next_index` (gone with the bridge's own registry) — the shape a real chain-14
    /// node with a bridged token sends.
    #[test]
    fn bridge_state_parses_the_nodes_own_pinned_shape() {
        let v = serde_json::json!({
            "enabled": true, "emitter": "01".repeat(32), "emitters": { "2": "02".repeat(32) },
            "guardian_set_index": 0, "guardians": ["aa".repeat(20)],
            "pq_guardians": ["cc".repeat(1312)],
            "mint_paused": false, "pause_nonce": 0, "list_nonce": 0,
            "pause_key": "dd".repeat(1312), "registration_fee": 1_000_000_000u64,
            "burn_sequence": 1,
            "assets": [{
                "index": 1, "chain": 2, "token": "aa".repeat(32), "asset_id": "bb".repeat(32),
                "decimals": 8, "locked": 600,
                "mint_cap_per_day": 100_000u64 * 100_000_000, "minted_today": 1_000, "mint_day": 0,
            }],
        });
        let state: BridgeState = serde_json::from_value(v).expect("must parse without next_index and with numeric amounts");
        assert_eq!(state.registration_fee, Some(1_000_000_000));
        assert_eq!(state.next_index, None);
        assert_eq!(state.assets[0].locked.as_deref(), Some("600"));
    }
}
