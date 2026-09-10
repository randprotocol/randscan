use crate::{error::AppError, state::AppState, ApiResult};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use randscan_core::{
    classify_query, PaginatedResponse, Pagination, PaginationInfo, ProgramDetail, ProgramSummary,
    QueryKind,
};
use randscan_db as db;

/// GET /api/v1/programs
pub async fn list_programs(
    State(state): State<AppState>,
    Query(p): Query<Pagination>,
) -> ApiResult<Json<PaginatedResponse<ProgramSummary>>> {
    let pool = state.db.inner();
    let rows = db::list_programs(pool, p.offset(), p.limit()).await?;
    let total = db::count_programs(pool).await?;
    Ok(Json(PaginatedResponse {
        data: rows.into_iter().map(Into::into).collect(),
        pagination: PaginationInfo::new(&p, total),
    }))
}

/// GET /api/v1/programs/:id
pub async fn get_program(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<ProgramDetail>> {
    let id = match classify_query(&id) {
        QueryKind::Hash(h) => h,
        _ => {
            return Err(AppError::BadRequest(
                "program id must be 64 hex characters".into(),
            ))
        }
    };
    let pool = state.db.inner();
    let row = db::get_program(pool, &id)
        .await?
        .ok_or_else(|| AppError::NotFound("program".into()))?;
    let calls = db::list_program_calls(pool, &id, 10).await?;
    Ok(Json(ProgramDetail {
        program: row.into(),
        recent_calls: calls.into_iter().map(Into::into).collect(),
    }))
}
