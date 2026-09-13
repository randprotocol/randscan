use crate::Result;
use sqlx::PgPool;

/// What the chain has seen of one bridged asset: how many attestations minted it and how many
/// burns sent it back out, with the public amounts summed in the asset's own smallest unit.
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

/// Per-asset totals of every bridge_attest and bridge_burn indexed so far, ordered by asset index.
/// A guardian-set rotation is a bridge_attest with no asset and is not counted.
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
