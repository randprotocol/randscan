//! Account handlers

use crate::{state::AppState, ApiResult};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use randscan_core::{
    AccountDetail, AccountTransaction, Pagination, PaginatedResponse, PaginationInfo,
    TokenAccountInfo,
};
use randscan_db as db;

/// GET /api/v1/accounts/:address
pub async fn get_account(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> ApiResult<Json<AccountDetail>> {
    let account = db::get_account_by_address(state.db.inner(), &address).await?;

    // Check if this is a validator
    let is_validator = db::validator_exists(state.db.inner(), &address)
        .await
        .unwrap_or(false);

    let stake_amount = if is_validator {
        db::get_validator_by_id(state.db.inner(), &address)
            .await
            .map(|v| Some(v.stake))
            .unwrap_or(None)
    } else {
        None
    };

    // Get token accounts
    let token_accounts = db::get_token_accounts_for_owner(state.db.inner(), &address)
        .await
        .unwrap_or_default();

    // Build token info by fetching mints
    let mut token_info: Vec<TokenAccountInfo> = Vec::new();
    for ta in token_accounts {
        if let Ok(mint) = db::get_token_mint(state.db.inner(), &ta.mint).await {
            token_info.push(TokenAccountInfo {
                token_mint: ta.mint.clone(),
                token_symbol: mint.symbol,
                balance: ta.balance,
                balance_display: ta.balance as f64 / 10_f64.powi(mint.decimals as i32),
                decimals: mint.decimals as i32,
            });
        }
    }

    Ok(Json(AccountDetail {
        address: account.address,
        atlas_balance: account.atlas_balance,
        atlas_balance_display: account.atlas_balance as f64 / 1_000_000_000.0,
        shrug_balance: account.shrug_balance,
        shrug_balance_display: account.shrug_balance as f64 / 1_000_000_000.0,
        nonce: account.nonce,
        is_executable: account.is_executable,
        owner: account.owner,
        data_len: account.data_len,
        tx_count: account.tx_count,
        first_seen: account.first_seen,
        last_seen: account.last_seen,
        is_validator,
        stake_amount,
        token_accounts: token_info,
    }))
}

/// GET /api/v1/accounts/:address/transactions
pub async fn get_account_transactions(
    State(state): State<AppState>,
    Path(address): Path<String>,
    Query(pagination): Query<Pagination>,
) -> ApiResult<Json<PaginatedResponse<AccountTransaction>>> {
    let offset = pagination.offset();
    let limit = pagination.limit();

    let transactions = db::get_account_transactions(state.db.inner(), &address, offset, limit).await?;

    let total = db::count_account_transactions(state.db.inner(), &address).await?;

    let results: Vec<AccountTransaction> = transactions
        .into_iter()
        .map(|at| AccountTransaction {
            account: at.account,
            tx_id: at.tx_id,
            role: at.role,
            block_height: at.block_height,
            timestamp: at.timestamp,
        })
        .collect();

    Ok(Json(PaginatedResponse {
        data: results,
        pagination: PaginationInfo::new(pagination.page, pagination.limit, total),
    }))
}

