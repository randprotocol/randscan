use crate::{state::AppState, ApiResult};
use axum::{
    extract::{Query, State},
    Json,
};
use randscan_core::{classify_query, QueryKind, SearchQuery, SearchResult, SearchResultType};
use randscan_db as db;

/// GET /api/v1/search?q=
///
/// A height finds a block; 64 hex characters find a block, a transaction, a program, a note
/// commitment or a nullifier; a base58 address finds a validator. There are no accounts to find.
pub async fn search(
    State(state): State<AppState>,
    Query(q): Query<SearchQuery>,
) -> ApiResult<Json<Vec<SearchResult>>> {
    let pool = state.db.inner();
    let mut results = Vec::new();

    match classify_query(&q.q) {
        QueryKind::Height(h) => {
            if let Some(b) = db::get_block_by_height(pool, h).await? {
                results.push(SearchResult {
                    result_type: SearchResultType::Block,
                    id: b.height.to_string(),
                    title: format!("Block #{}", b.height),
                    subtitle: Some(format!("{} transactions", b.tx_count)),
                    url: format!("/blocks/{}", b.height),
                });
            }
        }
        QueryKind::Hash(h) => {
            if let Some(b) = db::get_block_by_hash(pool, &h).await? {
                results.push(SearchResult {
                    result_type: SearchResultType::Block,
                    id: b.hash.clone(),
                    title: format!("Block #{}", b.height),
                    subtitle: Some(b.hash),
                    url: format!("/blocks/{}", b.height),
                });
            }
            if let Some(t) = db::get_transaction_summary(pool, &h).await? {
                results.push(SearchResult {
                    result_type: SearchResultType::Transaction,
                    id: t.hash.clone(),
                    title: format!("{} transaction", t.kind),
                    subtitle: Some(format!("block #{}", t.height)),
                    url: format!("/transactions/{}", t.hash),
                });
            }
            if let Some(p) = db::get_program(pool, &h).await? {
                results.push(SearchResult {
                    result_type: SearchResultType::Program,
                    id: p.id.clone(),
                    title: "Program".into(),
                    subtitle: Some(format!(
                        "deployed at #{}, {} calls",
                        p.deployed_at_height, p.call_count
                    )),
                    url: format!("/programs/{}", p.id),
                });
            }
            if let Some(n) = db::get_note_by_cm(pool, &h).await? {
                results.push(SearchResult {
                    result_type: SearchResultType::Note,
                    id: n.cm.clone(),
                    title: format!("Note #{}", n.leaf_index),
                    subtitle: Some(format!("commitment created at #{}", n.height)),
                    url: format!("/notes/{}", n.cm),
                });
            } else if let Some(t) = db::find_transaction_by_commitment(pool, &h).await? {
                // The block is indexed but the leaf page has not been fetched yet.
                results.push(SearchResult {
                    result_type: SearchResultType::Note,
                    id: h.clone(),
                    title: "Note".into(),
                    subtitle: Some(format!("commitment created by {} at #{}", t.kind, t.height)),
                    url: format!("/transactions/{}", t.hash),
                });
            }
            if let Some(nf) = db::get_nullifier(pool, &h).await? {
                results.push(SearchResult {
                    result_type: SearchResultType::Nullifier,
                    id: nf.nullifier.clone(),
                    title: "Nullifier".into(),
                    subtitle: Some(format!("note spent at #{}", nf.height)),
                    url: format!("/transactions/{}", nf.tx_hash),
                });
            }
        }
        QueryKind::Address(a) => {
            if let Some(v) = db::get_validator(pool, &a).await? {
                results.push(SearchResult {
                    result_type: SearchResultType::Validator,
                    id: a.clone(),
                    title: if v.active {
                        "Validator"
                    } else {
                        "Validator (inactive)"
                    }
                    .into(),
                    subtitle: Some(format!("stake {}", v.stake)),
                    url: format!("/validators/{}", a),
                });
            }
        }
        QueryKind::Unknown => {}
    }

    Ok(Json(results))
}
