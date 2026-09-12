use crate::{error::AppError, state::AppState, ApiResult};
use axum::{
    extract::{Path, State},
    Json,
};
use randscan_core::{Validator, ValidatorDetail};
use randscan_db as db;

async fn active_stake(pool: &db::PgPool) -> ApiResult<u128> {
    Ok(db::total_stake(pool).await?.parse().unwrap_or(0))
}

/// GET /api/v1/validators — the register, in address order (active entries are the rotation).
pub async fn list_validators(State(state): State<AppState>) -> ApiResult<Json<Vec<Validator>>> {
    let pool = state.db.inner();
    let total = active_stake(pool).await?;
    let rows = db::list_validators(pool).await?;
    Ok(Json(
        rows.into_iter().map(|r| r.into_validator(total)).collect(),
    ))
}

/// GET /api/v1/validators/:address
pub async fn get_validator(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> ApiResult<Json<ValidatorDetail>> {
    let pool = state.db.inner();
    let row = db::get_validator(pool, &address)
        .await?
        .ok_or_else(|| AppError::NotFound("validator".into()))?;
    let total = active_stake(pool).await?;
    let recent = db::list_blocks(pool, 0, 10, Some(&address)).await?;
    Ok(Json(ValidatorDetail {
        validator: row.into_validator(total),
        recent_blocks: recent.into_iter().map(Into::into).collect(),
    }))
}
