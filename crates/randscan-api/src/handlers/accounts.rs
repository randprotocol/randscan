use crate::{error::AppError, state::AppState, ApiResult};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use randscan_core::{
    AccountDetail, AccountTransaction, PaginatedResponse, Pagination, PaginationInfo,
};
use randscan_db as db;

/// GET /api/v1/accounts/:address
pub async fn get_account(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> ApiResult<Json<AccountDetail>> {
    let pool = state.db.inner();
    let row = match db::get_account(pool, &address).await? {
        Some(r) => r,
        None => {
            // Not indexed yet: ask the node; only surface accounts that exist on chain.
            let acc = state
                .indexer
                .rpc()
                .account(&address)
                .await
                .map_err(|_| AppError::NotFound("account".into()))?;
            if acc.balance == "0" && acc.nonce == 0 {
                return Err(AppError::NotFound("account".into()));
            }
            db::AccountRow {
                address: acc.address,
                balance: acc.balance,
                nonce: acc.nonce as i64,
                tx_count: 0,
                first_seen_height: 0,
                last_seen_height: 0,
            }
        }
    };
    let validator = db::get_validator(pool, &address).await?;
    let programs_deployed = db::count_programs_by_deployer(pool, &address).await?;
    Ok(Json(AccountDetail {
        address: row.address,
        balance: row.balance,
        nonce: row.nonce,
        tx_count: row.tx_count,
        first_seen_height: row.first_seen_height,
        last_seen_height: row.last_seen_height,
        is_validator: validator.is_some(),
        stake: validator.map(|v| v.stake),
        programs_deployed,
    }))
}

/// GET /api/v1/accounts/:address/transactions
pub async fn account_transactions(
    State(state): State<AppState>,
    Path(address): Path<String>,
    Query(p): Query<Pagination>,
) -> ApiResult<Json<PaginatedResponse<AccountTransaction>>> {
    let pool = state.db.inner();
    let rows = db::list_account_transactions(pool, &address, p.offset(), p.limit()).await?;
    let total = db::count_account_transactions(pool, &address).await?;
    Ok(Json(PaginatedResponse {
        data: rows
            .into_iter()
            .map(|r| AccountTransaction {
                summary: r.tx.into(),
                role: r.role,
            })
            .collect(),
        pagination: PaginationInfo::new(&p, total),
    }))
}
