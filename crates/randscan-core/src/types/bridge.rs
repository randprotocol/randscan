use serde::{Deserialize, Serialize};

/// A row of the bridge's asset registry (`shrugg_getAssets`). `index` is the `asset` word a note
/// of that asset carries; index 0 is SHRUGG and never appears here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BridgeAsset {
    pub index: i64,
    pub chain: i64,
    pub token: String,
    pub asset_id: String,
}

/// The bridge's public state (`shrugg_getBridgeState`). Bridged value is notes, so there are no
/// balances here; a chain without a bridge section reports `enabled: false` and nothing else.
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
}
