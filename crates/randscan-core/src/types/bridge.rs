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
