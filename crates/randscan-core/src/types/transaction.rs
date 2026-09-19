use serde::{Deserialize, Serialize};

/// Transaction kinds of the shielded RAND chain (`fullnode/docs/rpc.md`, `rand_getTransaction`).
///
/// Every transaction is a shielded bundle plus an *action*; the kind is the action's. The node's
/// `none` action (a plain shielded transfer, of RAND or of any RPL token — the asset is private)
/// is served as `transfer`. `Other` is any kind the node serves that this build does not decode;
/// the indexer keeps the node's tag in the database so a later build can backfill without
/// re-indexing.
///
/// Chain 14 (the hidden-asset bundle, RPL tokens, bridge hardening): there is no `token_transfer`
/// — a transfer of any asset is `none`, indistinguishable from a RAND payment. Eight kinds are
/// new: `register_token`, `token_mint`, `set_authority`, `token_burn` (RPL, spec §4/§6) and
/// `pause_mints`, `unpause_mints`, `register_bridged_token`, `list_backing` (bridge hardening
/// B1/B4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TxKind {
    Transfer,
    Mint,
    Deploy,
    Call,
    Bond,
    Unbond,
    Withdraw,
    BridgeAttest,
    BridgeBurn,
    RegisterToken,
    TokenMint,
    SetAuthority,
    TokenBurn,
    PauseMints,
    UnpauseMints,
    RegisterBridgedToken,
    ListBacking,
    Other,
}

impl TxKind {
    /// Kinds an API client may filter by (`GET /transactions?kind=`).
    pub const FILTERABLE: &'static [TxKind] = &[
        TxKind::Transfer,
        TxKind::Mint,
        TxKind::Deploy,
        TxKind::Call,
        TxKind::Bond,
        TxKind::Unbond,
        TxKind::Withdraw,
        TxKind::BridgeAttest,
        TxKind::BridgeBurn,
        TxKind::RegisterToken,
        TxKind::TokenMint,
        TxKind::SetAuthority,
        TxKind::TokenBurn,
        TxKind::PauseMints,
        TxKind::UnpauseMints,
        TxKind::RegisterBridgedToken,
        TxKind::ListBacking,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            TxKind::Transfer => "transfer",
            TxKind::Mint => "mint",
            TxKind::Deploy => "deploy",
            TxKind::Call => "call",
            TxKind::Bond => "bond",
            TxKind::Unbond => "unbond",
            TxKind::Withdraw => "withdraw",
            TxKind::BridgeAttest => "bridge_attest",
            TxKind::BridgeBurn => "bridge_burn",
            TxKind::RegisterToken => "register_token",
            TxKind::TokenMint => "token_mint",
            TxKind::SetAuthority => "set_authority",
            TxKind::TokenBurn => "token_burn",
            TxKind::PauseMints => "pause_mints",
            TxKind::UnpauseMints => "unpause_mints",
            TxKind::RegisterBridgedToken => "register_bridged_token",
            TxKind::ListBacking => "list_backing",
            TxKind::Other => "other",
        }
    }

    /// Exact match on a known, filterable kind; `None` for anything else (including `other`).
    pub fn parse(s: &str) -> Option<Self> {
        Self::FILTERABLE.iter().copied().find(|k| k.as_str() == s)
    }

    /// Like [`parse`](Self::parse) but maps unrecognised tags to [`TxKind::Other`].
    pub fn parse_lossy(s: &str) -> Self {
        Self::parse(s).unwrap_or(TxKind::Other)
    }

    /// The node's own action tag for this kind (`none` for a transfer).
    pub fn node_tag(&self) -> &'static str {
        match self {
            TxKind::Transfer => "none",
            other => other.as_str(),
        }
    }

    /// Kinds whose `amount` is in a bridged asset's own unit rather than RAND units.
    pub fn amount_is_bridged(&self) -> bool {
        matches!(self, TxKind::BridgeAttest | TxKind::BridgeBurn)
    }

    /// Kinds authorised by a PQ (Dilithium2) guardian quorum rather than a bundle proof or a
    /// single validator/authority signature (bridge hardening B1/B3/B4) — `pq_signers` on
    /// [`TransactionDetail`] is set exactly for these and for `bridge_attest`.
    pub fn has_pq_signers(&self) -> bool {
        matches!(
            self,
            TxKind::BridgeAttest | TxKind::UnpauseMints | TxKind::RegisterBridgedToken | TxKind::ListBacking
        )
    }
}

impl std::fmt::Display for TxKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The public fields of the chain-14 hidden-asset bundle: four input slots and four output slots
/// (dummies included), so always four `nullifiers`, four `commitments` and four `envelope_len`.
///
/// **There is no `asset` field.** Slots 0–1 carry a private asset (RAND or any RPL token) and
/// slots 2–3 always RAND; nothing public says which asset slots 0–1 moved — a transfer of RAND
/// and a transfer of any RPL token are the same shape, field for field. `burn_a`/`burn_r`/
/// `burn_asset` are the bundle's only public statement about value leaving the pool: `burn_a` and
/// `burn_asset` are non-zero only on a `token_burn` or a `bridge_burn` (equal to the action's
/// `amount` and `asset`); `burn_r` is RAND burned, non-zero only on a `bond` or a
/// `register_aggregator`. `fee` is always paid in RAND, from slots 2–3.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bundle {
    pub anchor: String,
    pub nullifiers: [String; 4],
    pub commitments: [String; 4],
    /// Units of RAND, as a decimal string.
    pub fee: String,
    /// The private asset burned (units of that asset's own smallest unit), decimal string.
    pub burn_a: String,
    /// RAND burned, decimal string.
    pub burn_r: String,
    /// 0 when nothing was burned; otherwise the registry index of the asset `burn_a` names.
    pub burn_asset: i64,
    /// Block height the sender targeted.
    pub time: i64,
    pub proof_len: i64,
    pub envelope_len: [i64; 4],
}

/// Transaction as shown in lists and pushed over WebSocket. Amounts are strings of units.
///
/// There is no sender, recipient or nonce: a shielded transaction has none. `amount` is the one
/// public amount an action carries (a deposit's note value, a staking move, or an RPL
/// registration/mint/burn); `validator` and `program` name what the action touched. `asset_index`
/// is the token registry index for a bridge or RPL action (`bridge_attest`, `bridge_burn`,
/// `token_mint`, `token_burn`, and the index a `register_token`/`list_backing` was given) — the
/// key a token list resolves to a symbol; `null` when this kind has none, or a
/// `bridge_attest`/rotation that deposits nothing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSummary {
    pub hash: String,
    pub height: i64,
    pub block_hash: String,
    pub tx_index: i32,
    pub kind: TxKind,
    /// RAND fee paid by the bundle ("0" for a validator-signed or bundle-less action).
    pub fee: String,
    pub timestamp_ms: i64,
    /// False for mint / unbond / withdraw / pause_mints / unpause_mints / register_bridged_token /
    /// list_backing, which carry no bundle (signed instead, some fee-less).
    pub has_bundle: bool,
    /// deploy: deployed program id; call: called program id
    pub program: Option<String>,
    /// bond / unbond / withdraw: the validator address; mint: the minting validator
    pub validator: Option<String>,
    /// mint, bond, unbond, withdraw: RAND units; bridge_attest, bridge_burn: bridged units;
    /// token_mint, token_burn: the token's own units; register_token: its initial mint's amount
    pub amount: Option<String>,
    /// the token registry index this action names (see the struct doc)
    pub asset_index: Option<i64>,
}

/// Receipt of a confidential call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    pub tx: String,
    pub program: String,
    pub tier: i32,
    pub outputs: Vec<i64>,
    pub height: i64,
    pub index: i32,
    /// The proof's salted commitment to the call's private inputs (zkVM M4.1).
    pub h_in: String,
}

/// An RPL mint authority (`register_token`'s `authority`, `set_authority`'s target): `"none"`
/// (fixed supply, or renounced), `"key"`, `"bridge"` or `"program"` — the tag alone, as
/// `rand_getTransaction`'s `tx_json` renders it (the full authority, with its key or backings, is
/// [`crate::TokenAuthority`], `rand_getTokens`'/`rand_getToken`'s).
pub type AuthorityKind = String;

/// The RPL initial mint a `register_token` may carry (`register_token`'s `initial` field) — every
/// word of the note the chain computes for it, so a recipient (or this indexer) rebuilds it with
/// nothing decrypted, whatever envelope the registrant published (`docs/bridge.md` §8's argument,
/// one action over).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InitialMint {
    pub amount: String,
    pub recipient: String,
    pub time: i64,
    pub r: String,
}

/// The RPL token actions' public fields (spec §4/§6): a token's registration and mints are public
/// by design, as a bridge deposit is — only a later *transfer* of the token's notes is shielded.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TokenAction {
    RegisterToken {
        name: String,
        symbol: String,
        decimals: i32,
        authority: AuthorityKind,
        /// The registry index this registration was given.
        index: i64,
        /// `null` for a registration with no initial mint.
        initial_amount: Option<String>,
        initial: Option<InitialMint>,
    },
    TokenMint {
        asset: i64,
        amount: String,
        recipient: String,
        time: i64,
        r: String,
        nonce: i64,
    },
    SetAuthority {
        asset: i64,
        nonce: i64,
        /// `null` is a renunciation: the token can never be minted again.
        new_authority: Option<String>,
    },
    /// A holder burn: public by design, since it is what makes `total_supply` auditable (a
    /// transfer of the same token is a plain `none` bundle, its asset private).
    TokenBurn { asset: i64, amount: String },
}

/// Bridge-hardening B1/B4's governance actions: bundle-less, and — apart from the pause key's own
/// `pause_mints` — authorised by the PQ guardian quorum, like a `bridge_attest`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BridgeGovernanceAction {
    /// The genesis pause key's own signature; no PQ quorum (only *lifting* the pause needs one).
    PauseMints { nonce: i64 },
    UnpauseMints { nonce: i64, pq_signers: Vec<i64> },
    RegisterBridgedToken {
        name: String,
        symbol: String,
        salt: String,
        chain: i32,
        token: String,
        decimals: i32,
        nonce: i64,
        asset_id: String,
        pq_signers: Vec<i64>,
    },
    ListBacking {
        token_index: i64,
        chain: i32,
        token: String,
        decimals: i32,
        nonce: i64,
        pq_signers: Vec<i64>,
    },
}

/// Full transaction detail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionDetail {
    #[serde(flatten)]
    pub summary: TransactionSummary,
    pub chain_id: i64,
    /// The bundle; `null` for a bundle-less signed action.
    pub bundle: Option<Bundle>,
    /// deploy: program length in words
    pub words_len: Option<i64>,
    /// call: size of the call's own proof
    pub call_proof_len: Option<i64>,
    /// call: size of the sealed input transcript, `null` when the caller published none
    pub input_envelope_len: Option<i64>,
    pub receipt: Option<Receipt>,
    /// mint: the deposit note's commitment
    pub cm: Option<String>,
    /// bond: whether this bond registered a new validator
    pub registered: Option<bool>,
    /// unbond / withdraw / token_mint / set_authority: the nonce it signed
    pub action_nonce: Option<i64>,
    /// bridge_attest: size of the guardian-signed message
    pub attestation_len: Option<i64>,
    /// bridge_attest / token_mint / register_token (initial mint): the depositor's or minter's
    /// recipient shielded address (`rand1…`)
    pub recipient: Option<String>,
    /// bridge_attest / token_mint / register_token (initial mint): the deposit/minted note's
    /// `time` word
    pub note_time: Option<i64>,
    /// relayer fee in bridged units (bridge_burn)
    pub relayer_fee: Option<String>,
    /// bridge_burn: destination chain id (1 Rand, 2 Ethereum, 3 BSC, 4 Tron, 5 Solana)
    pub to_chain: Option<i32>,
    /// bridge_burn: destination address, 32 bytes hex (EVM/Tron addresses left-padded)
    pub bridge_to: Option<String>,
    /// bridge_burn: the coin being redeemed (a backing's source-chain token address, 32 bytes hex)
    pub bridge_token: Option<String>,
    /// bridge_attest / token_mint / register_token (initial mint): the note's blinding, hex —
    /// public (a field of the signed action), and with `recipient`/`amount`/`note_time`/
    /// `asset_index` every word of the note, so it can be rebuilt with nothing decrypted.
    pub deposit_r: Option<String>,
    /// bridge_attest only: the leaf the chain appended for the deposit — the one commitment the
    /// wire does not carry, since the chain computes it. `null` for a rotation and for every
    /// other kind (a `token_mint`'s or a `register_token`'s initial mint's commitment is not
    /// published directly; look it up by `deposit_r`/`recipient`/`amount`/`note_time` instead, or
    /// find the note among this transaction's created notes).
    pub commitment: Option<String>,
    /// bridge_attest / unpause_mints / register_bridged_token / list_backing: the PQ guardians
    /// who co-signed, by index.
    pub pq_signers: Option<Vec<i64>>,
    /// register_token / token_mint / set_authority / token_burn.
    pub token_action: Option<TokenAction>,
    /// pause_mints / unpause_mints / register_bridged_token / list_backing.
    pub bridge_governance: Option<BridgeGovernanceAction>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_round_trip_through_serde_and_parse() {
        for k in TxKind::FILTERABLE {
            let json = serde_json::to_string(k).unwrap();
            assert_eq!(json, format!("\"{}\"", k.as_str()));
            assert_eq!(TxKind::parse(k.as_str()), Some(*k));
            assert_eq!(serde_json::from_str::<TxKind>(&json).unwrap(), *k);
        }
        assert_eq!(
            serde_json::to_string(&TxKind::BridgeBurn).unwrap(),
            "\"bridge_burn\""
        );
        assert_eq!(TxKind::Transfer.node_tag(), "none");
        assert_eq!(TxKind::Withdraw.node_tag(), "withdraw");
        assert_eq!(TxKind::RegisterBridgedToken.as_str(), "register_bridged_token");
    }

    #[test]
    fn there_is_no_token_transfer_kind() {
        assert_eq!(TxKind::parse("token_transfer"), None);
        assert_eq!(TxKind::parse_lossy("token_transfer"), TxKind::Other);
    }

    #[test]
    fn unknown_tags_are_other_only_when_lossy() {
        assert_eq!(
            TxKind::parse("none"),
            None,
            "the node tag is not an API filter"
        );
        assert_eq!(TxKind::parse("other"), None);
        assert_eq!(TxKind::parse_lossy("shielded"), TxKind::Other);
        assert_eq!(TxKind::parse_lossy("bond"), TxKind::Bond);
    }

    #[test]
    fn rpl_token_amounts_are_not_flagged_as_bridged() {
        // TokenMint/TokenBurn amounts are in the token's own unit (its registered `decimals`,
        // resolved through the token registry by `asset_index` — `formatTokenAmount` on the
        // frontend does this directly from `asset_index`, with no need for a `TxKind`-keyed
        // predicate), but they are not bridged: a native RPL token has no source chain.
        assert!(!TxKind::TokenMint.amount_is_bridged());
        assert!(!TxKind::TokenBurn.amount_is_bridged());
        assert!(!TxKind::Transfer.amount_is_bridged());
    }

    #[test]
    fn pq_signers_flags_the_guardian_authorised_kinds_only() {
        assert!(TxKind::BridgeAttest.has_pq_signers());
        assert!(TxKind::UnpauseMints.has_pq_signers());
        assert!(TxKind::RegisterBridgedToken.has_pq_signers());
        assert!(TxKind::ListBacking.has_pq_signers());
        assert!(!TxKind::PauseMints.has_pq_signers(), "the pause key alone, no quorum");
        assert!(!TxKind::TokenMint.has_pq_signers());
    }

    fn empty_summary(kind: TxKind) -> TransactionSummary {
        TransactionSummary {
            hash: "ab".into(),
            height: 1,
            block_hash: "cd".into(),
            tx_index: 0,
            kind,
            fee: "1000000".into(),
            timestamp_ms: 1,
            has_bundle: true,
            program: None,
            validator: None,
            amount: None,
            asset_index: None,
        }
    }

    fn empty_detail(kind: TxKind) -> TransactionDetail {
        TransactionDetail {
            summary: empty_summary(kind),
            chain_id: 14,
            bundle: None,
            words_len: None,
            call_proof_len: None,
            input_envelope_len: None,
            receipt: None,
            cm: None,
            registered: None,
            action_nonce: None,
            attestation_len: None,
            recipient: None,
            note_time: None,
            relayer_fee: None,
            to_chain: None,
            bridge_to: None,
            bridge_token: None,
            deposit_r: None,
            commitment: None,
            pq_signers: None,
            token_action: None,
            bridge_governance: None,
        }
    }

    #[test]
    fn detail_flattens_the_summary_and_has_a_four_slot_bundle_with_no_asset() {
        let mut d = empty_detail(TxKind::Transfer);
        d.bundle = Some(Bundle {
            anchor: "00".into(),
            nullifiers: ["01".into(), "02".into(), "03".into(), "04".into()],
            commitments: ["05".into(), "06".into(), "07".into(), "08".into()],
            fee: "1000000".into(),
            burn_a: "0".into(),
            burn_r: "0".into(),
            burn_asset: 0,
            time: 5,
            proof_len: 302857,
            envelope_len: [1380, 1380, 1380, 1380],
        });
        let v = serde_json::to_value(&d).unwrap();
        assert_eq!(v["kind"], "transfer");
        assert_eq!(v["bundle"]["nullifiers"].as_array().unwrap().len(), 4);
        assert_eq!(v["bundle"]["commitments"][3], "08");
        assert!(v["bundle"].get("asset").is_none(), "no public asset field on the bundle");
        assert!(v.get("sender").is_none() && v.get("nonce").is_none());
    }

    #[test]
    fn a_public_transfer_page_carries_no_asset_anywhere() {
        // A plain transfer's whole JSON: nothing that names which asset moved. This is the load
        // -bearing privacy property (task S1 item 3): the public page cannot know the asset.
        let mut d = empty_detail(TxKind::Transfer);
        d.bundle = Some(Bundle {
            anchor: "00".into(),
            nullifiers: ["01".into(), "02".into(), "03".into(), "04".into()],
            commitments: ["05".into(), "06".into(), "07".into(), "08".into()],
            fee: "1000000".into(),
            burn_a: "0".into(),
            burn_r: "0".into(),
            burn_asset: 0,
            time: 5,
            proof_len: 1,
            envelope_len: [1, 1, 1, 1],
        });
        let v = serde_json::to_value(&d).unwrap();
        let dump = v.to_string();
        assert!(!dump.contains("\"asset\":"), "a transfer's asset must never be serialised: {dump}");
        assert_eq!(v["asset_index"], serde_json::Value::Null);
    }

    #[test]
    fn token_mint_action_serialises_with_asset_units_needing_the_registry() {
        let mut d = empty_detail(TxKind::TokenMint);
        d.summary.amount = Some("700".into());
        d.summary.asset_index = Some(3);
        d.token_action = Some(TokenAction::TokenMint {
            asset: 3,
            amount: "700".into(),
            recipient: "rand1abc".into(),
            time: 41,
            r: "aa".repeat(32),
            nonce: 0,
        });
        let v = serde_json::to_value(&d).unwrap();
        assert_eq!(v["token_action"]["kind"], "token_mint");
        assert_eq!(v["token_action"]["asset"], 3);
        assert_eq!(v["asset_index"], 3);
    }

    #[test]
    fn register_token_carries_an_optional_initial_mint() {
        let mut d = empty_detail(TxKind::RegisterToken);
        d.token_action = Some(TokenAction::RegisterToken {
            name: "zUSD".into(),
            symbol: "zUSD".into(),
            decimals: 6,
            authority: "bridge".into(),
            index: 3,
            initial_amount: Some("5000".into()),
            initial: Some(InitialMint {
                amount: "5000".into(),
                recipient: "rand1abc".into(),
                time: 40,
                r: "bb".repeat(32),
            }),
        });
        let v = serde_json::to_value(&d).unwrap();
        assert_eq!(v["token_action"]["kind"], "register_token");
        assert_eq!(v["token_action"]["initial"]["amount"], "5000");
    }

    #[test]
    fn bridge_governance_actions_serialise_their_pq_signers() {
        let mut d = empty_detail(TxKind::UnpauseMints);
        d.bridge_governance = Some(BridgeGovernanceAction::UnpauseMints {
            nonce: 4,
            pq_signers: vec![0, 2],
        });
        let v = serde_json::to_value(&d).unwrap();
        assert_eq!(v["bridge_governance"]["pq_signers"], serde_json::json!([0, 2]));
    }
}
