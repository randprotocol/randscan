//! Token handlers

use crate::{error::AppError, state::AppState, ApiResult};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use randscan_core::{Pagination, PaginatedResponse, PaginationInfo, TokenMint, TokenSupply};
use randscan_db as db;

/// GET /api/v1/tokens
pub async fn get_tokens(
    State(state): State<AppState>,
    Query(pagination): Query<Pagination>,
) -> ApiResult<Json<PaginatedResponse<TokenMint>>> {
    let offset = pagination.offset();
    let limit = pagination.limit();

    let tokens = db::get_token_mints(state.db.inner(), offset, limit).await?;
    let total = db::count_token_mints(state.db.inner()).await?;

    let results: Vec<TokenMint> = tokens
        .into_iter()
        .map(|t| TokenMint {
            mint_address: t.mint_address,
            symbol: t.symbol,
            name: t.name,
            decimals: t.decimals,
            total_supply: t.total_supply,
            circulating_supply: t.circulating_supply,
            burned: t.burned,
            holder_count: t.holder_count,
            tx_count: t.tx_count,
            created_at: t.created_at.timestamp_millis(),
            updated_at: t.updated_at.timestamp_millis(),
        })
        .collect();

    Ok(Json(PaginatedResponse {
        data: results,
        pagination: PaginationInfo::new(pagination.page, pagination.limit, total),
    }))
}

/// GET /api/v1/tokens/:mint
pub async fn get_token(
    State(state): State<AppState>,
    Path(mint): Path<String>,
) -> ApiResult<Json<TokenSupply>> {
    let token = db::get_token_mint(state.db.inner(), &mint).await?;

    let decimals_factor = 10_f64.powi(token.decimals as i32);

    Ok(Json(TokenSupply {
        token: token.symbol,
        total_supply: token.total_supply,
        total_supply_display: token.total_supply as f64 / decimals_factor,
        circulating_supply: token.circulating_supply,
        circulating_supply_display: token.circulating_supply as f64 / decimals_factor,
        burned: token.burned,
        burned_display: token.burned as f64 / decimals_factor,
        decimals: token.decimals,
        holder_count: token.holder_count,
    }))
}
