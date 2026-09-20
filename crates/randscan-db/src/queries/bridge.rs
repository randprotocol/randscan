use crate::Result;
use sqlx::PgPool;

/// What the chain has seen of one bridged *token* (a registry index): how many attestations
/// minted it and how many burns sent it back out, with the public amounts summed in the token's
/// own smallest unit. Since chain 14 a token may have several backings (source coins), so this
/// is not one backing's figure — see [`bridge_backing_burns`].
#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct BridgeAssetFlowRow {
    pub asset_index: i64,
    pub deposits: i64,
    /// decimal string of units, "0" when there were no deposits
    pub deposited: String,
    pub burns: i64,
    pub burned: String,
    pub first_height: Option<i64>,
    pub last_height: Option<i64>,
}

/// Per-token totals of every bridge_attest and bridge_burn indexed so far, ordered by registry
/// index. A guardian-set rotation is a bridge_attest with no asset and is not counted.
pub async fn bridge_asset_flows(pool: &PgPool) -> Result<Vec<BridgeAssetFlowRow>> {
    Ok(sqlx::query_as::<_, BridgeAssetFlowRow>(
        "SELECT asset_index,
                COUNT(*) FILTER (WHERE kind = 'bridge_attest') AS deposits,
                COALESCE(SUM(amount) FILTER (WHERE kind = 'bridge_attest'), 0)::text AS deposited,
                COUNT(*) FILTER (WHERE kind = 'bridge_burn') AS burns,
                COALESCE(SUM(amount) FILTER (WHERE kind = 'bridge_burn'), 0)::text AS burned,
                MIN(height) AS first_height,
                MAX(height) AS last_height
         FROM transactions
         WHERE kind IN ('bridge_attest', 'bridge_burn') AND asset_index IS NOT NULL
         GROUP BY asset_index
         ORDER BY asset_index",
    )
    .fetch_all(pool)
    .await?)
}

/// The burns that redeemed one backing: a burn names the coin it releases as `(to_chain, token)`,
/// which the ledger holds to one of the token's backings, so a burn belongs to exactly one.
#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct BridgeBackingBurnRow {
    pub asset_index: i64,
    pub chain: i64,
    /// the backing's token address, 32 bytes hex, as the burn published it
    pub token: String,
    pub burns: i64,
    pub burned: String,
}

/// Per-backing totals of every bridge_burn indexed so far. There is no such query for deposits:
/// a bridge_attest publishes the token it minted (`asset_index`) and not the coin that was locked
/// for it, which is inside the attestation bytes the node serves by length only.
pub async fn bridge_backing_burns(pool: &PgPool) -> Result<Vec<BridgeBackingBurnRow>> {
    Ok(sqlx::query_as::<_, BridgeBackingBurnRow>(
        "SELECT asset_index, to_chain::bigint AS chain, LOWER(bridge_token) AS token,
                COUNT(*) AS burns, COALESCE(SUM(amount), 0)::text AS burned
         FROM transactions
         WHERE kind = 'bridge_burn' AND asset_index IS NOT NULL
               AND to_chain IS NOT NULL AND bridge_token IS NOT NULL
         GROUP BY asset_index, to_chain, LOWER(bridge_token)",
    )
    .fetch_all(pool)
    .await?)
}
