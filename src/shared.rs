use serde::{Serialize, Deserialize};

/// Which side of the connection a message originated from.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    Client,
    Server,
}

/// A `u8` value paired with its human-readable name (e.g. a TLS alert level).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NamedU8 {
    pub value: u8,
    pub name: String,
}

/// A `u16` value paired with its human-readable name (e.g. a TLS cipher suite).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NamedU16 {
    pub value: u16,
    pub name: String,
}

/// A `u32` value paired with its human-readable name (e.g. an HTTP/2 error code).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NamedU32 {
    pub value: u32,
    pub name: String,
}