//! The RPL token registry (`rand_getTokens` / `rand_getToken` / `rand_getTokenSupply`, chain 14
//! spec §4/§6). Everything here is public registry state — nothing says which notes hold a token;
//! that stays private in the shielded pool, disclosed only by a viewing key or a transaction key
//! (`TokenAction` on `TransactionDetail`, in `transaction.rs`, is the per-transaction half of this
//! same public registry).

use serde::{Deserialize, Serialize};

/// A source-chain coin behind a bridged token (`rand_getTokens`' `authority.backings`,
/// `rand_getTokenSupply`'s `backings`): its chain, its source token address, its **source**
/// decimals (the token itself is always eight decimals on Rand) and what this backing has locked
/// for/minted through it. `minted_today`/`mint_day`/`mint_cap_per_day` are bridge hardening B1 —
/// `None` on a node or a token predating it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenBacking {
    pub chain: i64,
    pub token: String,
    pub decimals: i32,
    pub locked: String,
    #[serde(default)]
    pub minted_today: Option<String>,
    #[serde(default)]
    pub mint_day: Option<i64>,
    #[serde(default)]
    pub mint_cap_per_day: Option<String>,
}

/// A token's mint authority in full (`rand_getTokens`/`rand_getToken`'s `authority`) — contrast
/// [`crate::AuthorityKind`], the bare tag `tx_json` renders on an action.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TokenAuthority {
    /// Fixed supply, or renounced: this token can never be minted again.
    None,
    Key { key: String, address: String },
    Bridge { backings: Vec<TokenBacking> },
    Program { program: String },
}

impl TokenAuthority {
    pub fn tag(&self) -> &'static str {
        match self {
            TokenAuthority::None => "none",
            TokenAuthority::Key { .. } => "key",
            TokenAuthority::Bridge { .. } => "bridge",
            TokenAuthority::Program { .. } => "program",
        }
    }
}

/// One registry row (`rand_getTokens` lists them, `rand_getToken` returns one). `index` is the
/// `asset` word a note of this token carries — 0 is RAND and never appears here. `id_text` is the
/// checksummed `rpl1…` text form (bech32m, HRP `rpl`, 62 characters).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenInfo {
    pub index: i64,
    pub id: String,
    pub id_text: String,
    pub name: String,
    pub symbol: String,
    pub decimals: i32,
    pub authority: TokenAuthority,
    pub mint_nonce: i64,
    pub total_supply: String,
    pub registered_at: i64,
}

impl TokenInfo {
    /// The backings behind this token, empty for a native (non-bridged) one.
    pub fn backings(&self) -> &[TokenBacking] {
        match &self.authority {
            TokenAuthority::Bridge { backings } => backings,
            _ => &[],
        }
    }
}

/// `rand_getTokens`' whole-registry page. `Default` is the disabled/empty placeholder a chain
/// without a bridge, a node predating this method, or the indexer's own not-yet-refreshed cache
/// all report identically.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TokenList {
    pub enabled: bool,
    /// A number, like `rand_getBridgeState`'s (`docs/rpc.md`: "`registration_fee` and
    /// `next_index` are numbers").
    #[serde(default)]
    pub registration_fee: Option<i64>,
    #[serde(default)]
    pub next_index: Option<i64>,
    pub tokens: Vec<TokenInfo>,
}

/// `rand_getTokenSupply`'s result: a token's public supply and, for a bridged token, each
/// backing's locked amount (the `locked` amounts sum to `total_supply`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenSupply {
    pub total_supply: String,
    #[serde(default)]
    pub backings: Vec<TokenBacking>,
}

/// One point of a token's public supply history, from an indexed `register_token` (the initial
/// mint), `token_mint` or `token_burn` — the RPL actions whose amount is public by design. `delta`
/// is signed: positive for a mint, negative for a burn, in the token's own smallest unit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenSupplyEvent {
    pub tx_hash: String,
    pub height: i64,
    pub timestamp_ms: i64,
    /// "register_token" | "token_mint" | "token_burn"
    pub kind: String,
    pub delta: String,
}

/// `/tokens/<rpl1…|index>`: the registry row plus its deploy transaction and supply history.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenDetail {
    #[serde(flatten)]
    pub info: TokenInfo,
    /// The `register_token` (or `register_bridged_token`) transaction that created this token.
    pub deploy_tx: Option<String>,
    /// Oldest first.
    pub supply_history: Vec<TokenSupplyEvent>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authority_tag_matches_the_variant() {
        assert_eq!(TokenAuthority::None.tag(), "none");
        assert_eq!(
            TokenAuthority::Key { key: "aa".into(), address: "2nRd".into() }.tag(),
            "key"
        );
        assert_eq!(TokenAuthority::Bridge { backings: vec![] }.tag(), "bridge");
        assert_eq!(TokenAuthority::Program { program: "aa".into() }.tag(), "program");
    }

    fn zusd() -> TokenInfo {
        TokenInfo {
            index: 3,
            id: "aa".repeat(32),
            id_text: "rpl1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqz".into(),
            name: "zUSD".into(),
            symbol: "zUSD".into(),
            decimals: 6,
            authority: TokenAuthority::Bridge {
                backings: vec![TokenBacking {
                    chain: 2,
                    token: "bb".repeat(32),
                    decimals: 6,
                    locked: "600".into(),
                    minted_today: Some("100".into()),
                    mint_day: Some(20345),
                    mint_cap_per_day: Some("10000000000".into()),
                }],
            },
            mint_nonce: 1,
            total_supply: "5700".into(),
            registered_at: 3,
        }
    }

    #[test]
    fn backings_is_empty_for_a_native_token_and_populated_for_a_bridged_one() {
        let native = TokenInfo { authority: TokenAuthority::None, ..zusd() };
        assert!(native.backings().is_empty());
        assert_eq!(zusd().backings().len(), 1);
        assert_eq!(zusd().backings()[0].minted_today.as_deref(), Some("100"));
    }

    #[test]
    fn token_list_round_trips_through_json() {
        let list = TokenList {
            enabled: true,
            registration_fee: Some(1_000_000_000),
            next_index: Some(4),
            tokens: vec![zusd()],
        };
        let v = serde_json::to_value(&list).unwrap();
        assert_eq!(v["tokens"][0]["symbol"], "zUSD");
        assert_eq!(v["tokens"][0]["authority"]["kind"], "bridge");
        assert_eq!(v["tokens"][0]["authority"]["backings"][0]["locked"], "600");
        let back: TokenList = serde_json::from_value(v).unwrap();
        assert_eq!(back, list);
    }

    #[test]
    fn a_disabled_chain_reports_no_tokens() {
        let v: TokenList = serde_json::from_value(serde_json::json!({ "enabled": false, "tokens": [] })).unwrap();
        assert!(!v.enabled);
        assert!(v.tokens.is_empty());
        assert_eq!(v.registration_fee, None);
    }
}
