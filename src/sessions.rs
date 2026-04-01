use std::net::IpAddr;
use std::ops::Deref;
use std::time::Duration;
use serde::{Serialize, Deserialize};
use ulid::Ulid;
use url::Url;
use chrono::{DateTime, Utc};
use crate::shared::NamedU16;
use crate::proxy_server::ProcessInfo;
use crate::serde_helpers::option_duration_millis;

/// A single HTTP transaction captured by the proxy.
///
/// Contains the full request/response pair, timing information, TLS details
/// and optional HTTP/2 stream reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionEntry {
    pub id: Ulid,
    pub url: Url,
    pub client_addr: Option<NetAddress>,
    pub remote_addr: Option<NetAddress>,
    pub http_version: String,
    pub transaction_type: TransactionType,
    pub request: Request,
    pub response: Option<Response>,
    #[serde(rename = "isWebSocket")]
    pub is_websocket: bool,
    pub tls: TlsSettings,
    pub http2: Option<Http2StreamRef>,
    pub timings: Timings,
    pub process: Option<ProcessInfo>,
}

/// An IP address with an optional port.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetAddress {
    pub ip: IpAddr,
    pub port: Option<u16>,
}

/// Whether the entry represents a regular request or an HTTP/2 server push.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    Request,
    PushPromise,
}

/// TLS connection metadata associated with a session entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TlsSettings {
    pub connection_id: Option<Ulid>,
    pub tls_version: Option<NamedU16>,
    pub cipher_suite: Option<NamedU16>,
    pub ja3: Option<Ja3>,
    pub ja4: Option<Ja4>,
}

/// Reference to an HTTP/2 stream within a connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Http2StreamRef {
    pub connection_id: Ulid,
    pub stream_id: u32,
}

/// Timing breakdown for a session entry.
///
/// Each phase is optional and will be `None` when the phase was not
/// observed or does not apply.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Timings {
    /// Request start (UTC).
    pub started_at: DateTime<Utc>,
    /// Duration in milliseconds.
    #[serde(with = "option_duration_millis")]
    pub blocked: Option<Duration>,
    /// Duration in milliseconds.
    #[serde(with = "option_duration_millis")]
    pub dns: Option<Duration>,
    /// Duration in milliseconds.
    #[serde(with = "option_duration_millis")]
    pub connect: Option<Duration>,
    /// Duration in milliseconds.
    #[serde(with = "option_duration_millis")]
    pub send: Option<Duration>,
    /// Duration in milliseconds.
    #[serde(with = "option_duration_millis")]
    pub wait: Option<Duration>,
    /// Duration in milliseconds.
    #[serde(with = "option_duration_millis")]
    pub receive: Option<Duration>,
    /// Duration in milliseconds.
    #[serde(with = "option_duration_millis")]
    pub ssl: Option<Duration>,
}

/// JA3 TLS fingerprint (raw string + MD5 hash).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ja3 {
    pub string: String,
    pub hash: String,
}

/// JA4 TLS fingerprint (raw + hashed form).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ja4 {
    pub raw: String,
    pub hashed: String,
}

/// Ordered list of HTTP headers.
///
/// Header name lookups are case-insensitive. Duplicate names are allowed,
/// use [`get_all`](Headers::get_all) to retrieve every value for a given name.
///
/// ```
/// use powhttp_sdk::Headers;
///
/// let mut headers = Headers::new();
/// headers.push("Content-Type", "application/json");
/// headers.push("Accept", "text/html");
///
/// assert_eq!(headers.get("content-type"), Some("application/json"));
/// assert_eq!(headers.len(), 2);
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Headers {
    inner: Vec<(String, String)>,
}

impl Headers {
    /// Creates an empty header list.
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    /// Returns the value of the first header matching `name` (case-insensitive).
    pub fn get(&self, name: &str) -> Option<&str> {
        let name_lower = name.to_lowercase();
        self.inner
            .iter()
            .find(|(name, _)| name.to_lowercase() == name_lower)
            .map(|(_, value)| value.as_str())
    }

    /// Returns all values for headers matching `name` (case-insensitive).
    pub fn get_all(&self, name: &str) -> Vec<&str> {
        let name_lower = name.to_lowercase();
        self.inner
            .iter()
            .filter(|(name, _)| name.to_lowercase() == name_lower)
            .map(|(_, value)| value.as_str())
            .collect()
    }

    /// Returns `true` if a header with the given `name` exists (case-insensitive).
    pub fn contains(&self, name: &str) -> bool {
        let name_lower = name.to_lowercase();
        self.inner.iter().any(|(name, _)| name.to_lowercase() == name_lower)
    }

    /// Sets the value of the first header matching `name` or appends a new one.
    pub fn set(&mut self, name: impl Into<String>, value: impl Into<String>) {
        let name = name.into();
        let name_lower = name.to_lowercase();
        if let Some((_, v)) = self.inner.iter_mut().find(|(name, _)| name.to_lowercase() == name_lower) {
            *v = value.into();
        } else {
            self.inner.push((name, value.into()));
        }
    }

    /// Appends a header even if one with the same name already exists.
    pub fn push(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.inner.push((name.into(), value.into()));
    }

    /// Removes and returns the value of the first header matching `name`.
    pub fn remove(&mut self, name: &str) -> Option<String> {
        let name_lower = name.to_lowercase();
        if let Some(pos) = self.inner.iter().position(|(name, _)| name.to_lowercase() == name_lower) {
            Some(self.inner.remove(pos).1)
        } else {
            None
        }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, (String, String)> {
        self.inner.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, (String, String)> {
        self.inner.iter_mut()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl Deref for Headers {
    type Target = [(String, String)];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl IntoIterator for Headers {
    type Item = (String, String);
    type IntoIter = std::vec::IntoIter<(String, String)>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<'a> IntoIterator for &'a Headers {
    type Item = &'a (String, String);
    type IntoIter = std::slice::Iter<'a, (String, String)>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

impl<'a> IntoIterator for &'a mut Headers {
    type Item = &'a mut (String, String);
    type IntoIter = std::slice::IterMut<'a, (String, String)>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter_mut()
    }
}

impl FromIterator<(String, String)> for Headers {
    fn from_iter<T: IntoIterator<Item = (String, String)>>(iter: T) -> Self {
        Self { 
            inner: Vec::from_iter(iter)
        }
    }
}

impl From<Vec<(String, String)>> for Headers {
    fn from(vec: Vec<(String, String)>) -> Self {
        Self { inner: vec }
    }
}

impl From<Headers> for Vec<(String, String)> {
    fn from(headers: Headers) -> Self {
        headers.inner
    }
}

impl Serialize for Headers {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.inner.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Headers {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Vec::<(String, String)>::deserialize(deserializer).map(|inner| Headers { inner })
    }
}

/// The request half of a session entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub method: Option<String>,
    pub path: Option<String>,
    pub http_version: Option<String>,
    pub headers: Headers,
    pub body_size: Option<usize>,
}

/// The response half of a session entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub http_version: Option<String>,
    pub status_code: Option<u16>,
    pub status_text: Option<String>,
    pub headers: Headers,
    pub body_size: Option<usize>,
}

/// Lightweight session descriptor returned by [`ExtensionHandle::list_sessions`](crate::ExtensionHandle::list_sessions).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    pub id: Ulid,
    pub name: String,
}