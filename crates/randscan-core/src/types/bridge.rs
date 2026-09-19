use serde::{Deserialize, Serialize};

/// A row of the bridge's asset registry (`rand_getAssets`, and `rand_getBridgeState.assets`).
/// `index` is the `asset` word a note of that asset carries; index 0 is RAND and never appears
/// here. `decimals`/`locked`/`minted_today`/`mint_day` are bridge hardening B1 — one row per
/// *backing* (a token with several source coins, spec §12, lists once per backing, all sharing
/// `index`); `None` on a node predating B1.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BridgeAsset {
    pub index: i64,
    pub chain: i64,
    pub token: String,
    pub asset_id: String,
    #[serde(default)]
    pub decimals: Option<i32>,
    #[serde(default)]
    pub locked: Option<String>,
    #[serde(default)]
    pub minted_today: Option<String>,
    #[serde(default)]
    pub mint_day: Option<i64>,
    /// The daily mint cap for this backing (bridge hardening B1).
    #[serde(default)]
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
    /// is a chain-wide constant, not a value that grows with usage the way a supply does.
    #[serde(default)]
    pub registration_fee: Option<i64>,
    #[serde(default)]
    pub burn_sequence: Option<i64>,
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
