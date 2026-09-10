use crate::{state::AppState, ApiResult};
use axum::{
    extract::{Query, State},
    Json,
};
use randscan_core::{
    classify_query, format_units, QueryKind, SearchQuery, SearchResult, SearchResultType,
};
use randscan_db as db;

/// GET /api/v1/search?q=
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
            if let Some(t) = db::get_transaction(pool, &h).await? {
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
                        "deployed at #{} by {}",
                        p.deployed_at_height, p.deployer
                    )),
                    url: format!("/programs/{}", p.id),
                });
            }
        }
        QueryKind::Address(a) => {
            let mut found = db::get_account(pool, &a).await?.map(|r| r.balance);
            if found.is_none() {
                if let Ok(acc) = state.indexer.rpc().account(&a).await {
                    if acc.balance != "0" || acc.nonce != 0 {
                        found = Some(acc.balance);
                    }
                }
            }
            if let Some(balance) = found {
                results.push(SearchResult {
                    result_type: SearchResultType::Account,
                    id: a.clone(),
                    title: "Account".into(),
                    subtitle: Some(format!("{} SHRUGG", format_units(&balance))),
                    url: format!("/account/{}", a),
                });
            }
            if let Some(v) = db::get_validator(pool, &a).await? {
                results.push(SearchResult {
                    result_type: SearchResultType::Validator,
                    id: a.clone(),
                    title: "Validator".into(),
                    subtitle: Some(format!("stake {}", v.stake)),
                    url: format!("/validators/{}", a),
                });
            }
        }
        QueryKind::Unknown => {}
    }

    Ok(Json(results))
}
