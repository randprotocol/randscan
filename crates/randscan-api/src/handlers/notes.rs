use crate::{error::AppError, state::AppState, ApiResult};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use randscan_core::{
    classify_query, Note, Nullifier, PaginatedResponse, Pagination, PaginationInfo, QueryKind,
};
use randscan_db as db;

/// GET /api/v1/notes — the commitment tree, newest leaf first.
pub async fn list_notes(
    State(state): State<AppState>,
    Query(p): Query<Pagination>,
) -> ApiResult<Json<PaginatedResponse<Note>>> {
    let pool = state.db.inner();
    let rows = db::list_notes(pool, p.offset(), p.limit()).await?;
    let total = db::count_notes(pool).await?;
    Ok(Json(PaginatedResponse {
        data: rows.into_iter().map(Into::into).collect(),
        pagination: PaginationInfo::new(&p, total),
    }))
}

/// GET /api/v1/notes/:id — a leaf by commitment (64 hex) or by leaf index (decimal).
pub async fn get_note(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<Note>> {
    let pool = state.db.inner();
    let row = match classify_query(&id) {
        QueryKind::Hash(cm) => db::get_note_by_cm(pool, &cm).await?,
        QueryKind::Height(index) => db::get_note_by_index(pool, index).await?,
        _ => {
            return Err(AppError::BadRequest(
                "note id must be a commitment (64 hex) or a leaf index".into(),
            ))
        }
    };
    Ok(Json(
        row.ok_or_else(|| AppError::NotFound("note".into()))?.into(),
    ))
}

/// GET /api/v1/nullifiers/:nf — the transaction that published this nullifier.
pub async fn get_nullifier(
    State(state): State<AppState>,
    Path(nf): Path<String>,
) -> ApiResult<Json<Nullifier>> {
    let nf = match classify_query(&nf) {
        QueryKind::Hash(h) => h,
        _ => {
            return Err(AppError::BadRequest(
                "nullifier must be 64 hex characters".into(),
            ))
        }
    };
    let row = db::get_nullifier(state.db.inner(), &nf)
        .await?
        .ok_or_else(|| AppError::NotFound("nullifier".into()))?;
    Ok(Json(row.into()))
}

/// The most nullifiers one `POST /api/v1/nullifiers/lookup` may name.
pub const MAX_NULLIFIER_LOOKUP: usize = 1000;

#[derive(Debug, serde::Deserialize)]
pub struct NullifierLookup {
    pub nullifiers: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct NullifierLookupResult {
    /// The published ones among those asked for; a nullifier absent here is unspent.
    pub spent: Vec<Nullifier>,
}

/// POST /api/v1/nullifiers/lookup — which of up to 1000 nullifiers are published. A history scan
/// asks about every note a key received; one request per note would spend a whole anonymous
/// rate-limit window on the first sixty.
pub async fn lookup_nullifiers(
    State(state): State<AppState>,
    Json(body): Json<NullifierLookup>,
) -> ApiResult<Json<NullifierLookupResult>> {
    if body.nullifiers.len() > MAX_NULLIFIER_LOOKUP {
        return Err(AppError::BadRequest(format!(
            "at most {} nullifiers per lookup",
            MAX_NULLIFIER_LOOKUP
        )));
    }
    let mut nfs = Vec::with_capacity(body.nullifiers.len());
    for nf in &body.nullifiers {
        match classify_query(nf) {
            QueryKind::Hash(h) => nfs.push(h),
            _ => {
                return Err(AppError::BadRequest(
                    "nullifier must be 64 hex characters".into(),
                ))
            }
        }
    }
    let rows = db::get_nullifiers(state.db.inner(), &nfs).await?;
    Ok(Json(NullifierLookupResult {
        spent: rows.into_iter().map(Into::into).collect(),
    }))
}
