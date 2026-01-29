//! Token types for RandScan

use serde::{Deserialize, Serialize};

/// Token identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TokenId {
    Atlas,
    Shrug,
}

impl TokenId {
    pub fn symbol(&self) -> &'static str {
        match self {
            TokenId::Atlas => "ATLAS",
            TokenId::Shrug => "SHRUG",
        }
    }

    pub fn decimals(&self) -> u8 {
        9
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "ATLAS" => Some(TokenId::Atlas),
            "SHRUG" => Some(TokenId::Shrug),
            _ => None,
        }
    }
}

/// Token mint information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMint {
    pub mint_address: String,
    pub symbol: String,
    pub name: String,
    pub decimals: i16,
    pub total_supply: i64,
    pub circulating_supply: i64,
    pub burned: i64,
    pub holder_count: i32,
    pub tx_count: i32,
    pub created_at: i64,
    pub updated_at: i64,
}

impl TokenMint {
    pub fn supply_display(&self) -> f64 {
        self.total_supply as f64 / 10_f64.powi(self.decimals as i32)
    }

    pub fn circulating_display(&self) -> f64 {
        self.circulating_supply as f64 / 10_f64.powi(self.decimals as i32)
    }

    pub fn burned_display(&self) -> f64 {
        self.burned as f64 / 10_f64.powi(self.decimals as i32)
    }
}

/// Token account (user balance of specific token)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAccount {
    pub account_address: String,
    pub owner: String,
    pub mint: String,
    pub balance: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

impl TokenAccount {
    pub fn balance_display(&self, decimals: i16) -> f64 {
        self.balance as f64 / 10_f64.powi(decimals as i32)
    }
}

/// Token transfer record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenTransfer {
    pub tx_id: String,
    pub mint: String,
    pub from_address: String,
    pub to_address: String,
    pub amount: i64,
    pub timestamp: i64,
}

/// Token supply response for API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSupply {
    pub token: String,
    pub total_supply: i64,
    pub total_supply_display: f64,
    pub circulating_supply: i64,
    pub circulating_supply_display: f64,
    pub burned: i64,
    pub burned_display: f64,
    pub decimals: i16,
    pub holder_count: i32,
}

/// Token holder for rich list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenHolder {
    pub address: String,
    pub balance: i64,
    pub balance_display: f64,
    pub percentage: f64,
    pub rank: i32,
}
