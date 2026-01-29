//! RandScan Core - Shared types for RandProtocol blockchain explorer
//!
//! This crate contains all the core types that mirror RandProtocol's blockchain
//! structures, plus additional types for the scanner's internal use.

pub mod types;
pub mod error;

pub use types::*;
pub use error::*;

/// Token decimals for ATLAS and SHRUG
pub const TOKEN_DECIMALS: u8 = 9;

/// Lamports per token (10^9)
pub const LAMPORTS_PER_TOKEN: u64 = 1_000_000_000;

/// Minimum stake in lamports (10,000 ATLAS)
pub const MINIMUM_STAKE: u64 = 10_000 * LAMPORTS_PER_TOKEN;

/// Convert lamports to display units
pub fn lamports_to_display(lamports: u64) -> f64 {
    lamports as f64 / LAMPORTS_PER_TOKEN as f64
}

/// Convert display units to lamports
pub fn display_to_lamports(display: f64) -> u64 {
    (display * LAMPORTS_PER_TOKEN as f64) as u64
}
