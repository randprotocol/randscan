use crate::{error::AppError, ApiResult};
use axum::{extract::Path, http::StatusCode, Json};

/// GET /api/v1/accounts/:address and /accounts/:address/transactions
///
/// The shielded chain has no accounts: balances are notes only a viewing key can open, and a
/// transaction names no sender or recipient. Answer 410 with a pointer rather than 404, so an
/// integration written against the account chain fails with a reason.
pub async fn no_accounts(Path(_address): Path<String>) -> ApiResult<Json<()>> {
    Err(AppError::Status {
        status: StatusCode::GONE,
        error: "no_accounts",
        message: "this chain has no accounts: balances are private notes and transactions carry \
                  no sender or recipient; see /validators for the public register and /notes \
                  for the commitment tree"
            .into(),
    })
}
