//! Core types for RandScan blockchain explorer

mod block;
mod transaction;
mod account;
mod validator;
mod token;
mod stats;
mod api;
mod ws;

pub use block::*;
pub use transaction::*;
pub use account::*;
pub use validator::*;
pub use token::*;
pub use stats::*;
pub use api::*;
pub use ws::*;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Type alias for view number in HotStuff consensus
pub type ViewNumber = u64;

/// Type alias for block height (slot number)
pub type Height = u64;

/// Type alias for epoch number
pub type Epoch = u64;

/// 32-byte identifier used for blocks, transactions, validators, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Id32(#[serde(with = "hex_serde")] pub [u8; 32]);

impl Id32 {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn zero() -> Self {
        Self([0u8; 32])
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    pub fn to_short_hex(&self) -> String {
        hex::encode(&self.0[..8])
    }

    pub fn from_hex(s: &str) -> Result<Self, hex::FromHexError> {
        let bytes = hex::decode(s)?;
        if bytes.len() != 32 {
            return Err(hex::FromHexError::InvalidStringLength);
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Self(arr))
    }

    pub fn is_zero(&self) -> bool {
        self.0 == [0u8; 32]
    }
}

impl Default for Id32 {
    fn default() -> Self {
        Self::zero()
    }
}

impl std::fmt::Display for Id32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_short_hex())
    }
}

/// 64-byte signature
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature64(#[serde(with = "hex_serde_64")] pub [u8; 64]);

impl Signature64 {
    pub fn new(bytes: [u8; 64]) -> Self {
        Self(bytes)
    }

    pub fn zero() -> Self {
        Self([0u8; 64])
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    pub fn from_hex(s: &str) -> Result<Self, hex::FromHexError> {
        let bytes = hex::decode(s)?;
        if bytes.len() != 64 {
            return Err(hex::FromHexError::InvalidStringLength);
        }
        let mut arr = [0u8; 64];
        arr.copy_from_slice(&bytes);
        Ok(Self(arr))
    }
}

impl Default for Signature64 {
    fn default() -> Self {
        Self::zero()
    }
}

/// 48-byte encrypted amount for private transactions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncryptedAmount(#[serde(with = "hex_serde_48")] pub [u8; 48]);

impl EncryptedAmount {
    pub fn new(bytes: [u8; 48]) -> Self {
        Self(bytes)
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    pub fn from_hex(s: &str) -> Result<Self, hex::FromHexError> {
        let bytes = hex::decode(s)?;
        if bytes.len() != 48 {
            return Err(hex::FromHexError::InvalidStringLength);
        }
        let mut arr = [0u8; 48];
        arr.copy_from_slice(&bytes);
        Ok(Self(arr))
    }
}

/// Compute SHA256 hash
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Hex serialization for 32-byte arrays
mod hex_serde {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8; 32], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let bytes = hex::decode(&s).map_err(serde::de::Error::custom)?;
        if bytes.len() != 32 {
            return Err(serde::de::Error::custom("Invalid length for 32-byte array"));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(arr)
    }
}

/// Hex serialization for 64-byte arrays
mod hex_serde_64 {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8; 64], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 64], D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let bytes = hex::decode(&s).map_err(serde::de::Error::custom)?;
        if bytes.len() != 64 {
            return Err(serde::de::Error::custom("Invalid length for 64-byte array"));
        }
        let mut arr = [0u8; 64];
        arr.copy_from_slice(&bytes);
        Ok(arr)
    }
}

/// Hex serialization for 48-byte arrays
mod hex_serde_48 {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8; 48], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 48], D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let bytes = hex::decode(&s).map_err(serde::de::Error::custom)?;
        if bytes.len() != 48 {
            return Err(serde::de::Error::custom("Invalid length for 48-byte array"));
        }
        let mut arr = [0u8; 48];
        arr.copy_from_slice(&bytes);
        Ok(arr)
    }
}
