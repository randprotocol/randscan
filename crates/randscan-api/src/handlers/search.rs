//! Search handler

use crate::{error::AppError, state::AppState, ApiResult};
use axum::{
    extract::{Query, State},
    Json,
};
use randscan_core::{SearchQuery, SearchResult, SearchResultType};
use randscan_db as db;

/// GET /api/v1/search?q=
pub async fn search(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> ApiResult<Json<Vec<SearchResult>>> {
    let q = query.q.trim();
    let limit = query.limit.unwrap_or(10) as i64;

    if q.is_empty() {
        return Ok(Json(Vec::new()));
    }

    let mut results = Vec::new();

    // Check if it looks like a block height
    if let Ok(height) = q.parse::<i64>() {
        if let Ok(block) = db::get_block_by_height(state.db.inner(), height).await {
            results.push(SearchResult {
                result_type: SearchResultType::Block,
                id: block.block_id.clone(),
                title: format!("Block #{}", block.height),
                subtitle: Some(format!("{} transactions", block.transaction_count)),
                url: format!("/block/{}", block.height),
            });
        }
    }

    // Search blocks by ID
    if results.is_empty() || q.len() >= 8 {
        if let Ok(block) = db::get_block_by_id(state.db.inner(), q).await {
            results.push(SearchResult {
                result_type: SearchResultType::Block,
                id: block.block_id.clone(),
                title: format!("Block #{}", block.height),
                subtitle: Some(format!("{} transactions", block.transaction_count)),
                url: format!("/block/{}", block.height),
            });
        }
    }

    // Search transactions
    let txs = db::search_transactions(state.db.inner(), q, limit).await?;
    for tx in txs {
        results.push(SearchResult {
            result_type: SearchResultType::Transaction,
            id: tx.tx_id.clone(),
            title: format!("Transaction {}", &tx.tx_id[..16.min(tx.tx_id.len())]),
            subtitle: Some(format!("{} - {}", tx.payload_type, tx.status)),
            url: format!("/tx/{}", tx.tx_id),
        });
    }

    // Search accounts
    let accounts = db::search_accounts(state.db.inner(), q, limit).await?;
    for account in accounts {
        results.push(SearchResult {
            result_type: SearchResultType::Account,
            id: account.address.clone(),
            title: format!(
                "Account {}",
                &account.address[..16.min(account.address.len())]
            ),
            subtitle: Some(format!("{} transactions", account.tx_count)),
            url: format!("/account/{}", account.address),
        });
    }

    // Search validators
    let validators = db::search_validators(state.db.inner(), q, limit).await?;
    for validator in validators {
        results.push(SearchResult {
            result_type: SearchResultType::Validator,
            id: validator.validator_id.clone(),
            title: format!(
                "Validator {}",
                &validator.validator_id[..16.min(validator.validator_id.len())]
            ),
            subtitle: Some(format!(
                "{} ATLAS staked",
                validator.stake / 1_000_000_000
            )),
            url: format!("/validator/{}", validator.validator_id),
        });
    }

    // Search tokens
    let tokens = db::search_tokens(state.db.inner(), q, limit).await?;
    for token in tokens {
        results.push(SearchResult {
            result_type: SearchResultType::Token,
            id: token.mint_address.clone(),
            title: format!("{} ({})", token.name, token.symbol),
            subtitle: Some(format!("{} holders", token.holder_count)),
            url: format!("/token/{}", token.mint_address),
        });
    }

    // Limit total results
    results.truncate(limit as usize);

    Ok(Json(results))
}
