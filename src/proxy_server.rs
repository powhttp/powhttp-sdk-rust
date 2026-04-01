use std::net::SocketAddr;
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use url::Url;
use crate::sessions::Headers;

/// Context passed to a connect handler when a client connects to the powhttp proxy server.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectContext {
    pub session_id: Ulid,
    pub target_addr: TargetAddress,
    pub client_addr: SocketAddr,
    pub process: Option<ProcessInfo>,
    pub request: ConnectRequest,
}

/// A host–port pair identifying the target server.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetAddress {
    pub host: String,
    pub port: u16,
}

impl TargetAddress {
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self { host: host.into(), port }
    }
}

/// Information about the process that initiated the connection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Socks4Request {
    pub user_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpConnectRequest {
    pub target: String,
    pub http_version: String,
    pub headers: Headers,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpDirectRequest {
    pub method: String,
    pub path: String,
    pub http_version: String,
    pub headers: Headers,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpProxyRequest {
    pub method: String,
    pub url: Url,
    pub http_version: String,
    pub headers: Headers,
}

/// The protocol-specific request that initiated the proxy connection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ConnectRequest {
    #[serde(rename = "socks4")]
    Socks4(Socks4Request),
    #[serde(rename = "socks5")]
    Socks5,
    HttpConnect(HttpConnectRequest),
    TlsSni,
    HttpDirect(HttpDirectRequest),
    HttpProxy(HttpProxyRequest),
}

/// Options returned when accepting a connection.
///
/// All fields default to `None`, which tells the proxy to use its own defaults.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptOptions {
    pub target_addr: Option<TargetAddress>,
    pub proxy: Option<Proxy>,
    pub record: Option<bool>,
    pub intercept_tls: Option<bool>,
}

/// The result returned by a connect handler to accept or reject the connection.
///
/// ```
/// use powhttp_sdk::proxy_server::ConnectResult;
///
/// let result = ConnectResult::accept();
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ConnectResult {
    Accept(AcceptOptions),
    Reject,
}

impl ConnectResult {
    /// Accept the connection with default settings.
    pub fn accept() -> Self {
        ConnectResult::Accept(AcceptOptions::default())
    }

    /// Accept the connection and route it through an upstream proxy.
    pub fn use_proxy(proxy: Option<Proxy>) -> Self {
        ConnectResult::Accept(
            AcceptOptions { proxy, ..Default::default() }
        )
    }
    
    /// Reject the connection.
    pub fn reject() -> Self {
        ConnectResult::Reject
    }
}

impl From<AcceptOptions> for ConnectResult {
    fn from(accept: AcceptOptions) -> Self {
        ConnectResult::Accept(accept)
    }
}

/// Upstream proxy protocol.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProxyType {
    #[default]
    Http,
    Https,
    Socks4,
    Socks4a,
    Socks5,
    Socks5h,
}

/// An upstream proxy to route connections through.
///
/// Use one of the constructor methods ([`http`](Proxy::http), [`socks5`](Proxy::socks5), etc.)
/// to create an instance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Proxy {
    proxy_type: ProxyType,
    host: String,
    port: u16,
    username: Option<String>,
    password: Option<String>,
}

impl Proxy {
    pub fn http(host: String, port: u16, username: Option<String>, password: Option<String>) -> Self {
        Self { 
            proxy_type: ProxyType::Http, 
            host: host.into(), 
            port, 
            username, 
            password 
        }
    }

    pub fn https(host: String, port: u16, username: Option<String>, password: Option<String>) -> Self {
        Self { 
            proxy_type: ProxyType::Https, 
            host: host.into(), 
            port, 
            username, 
            password 
        }
    }

    pub fn socks4(host: String, port: u16, user_id: Option<String>) -> Self {
        Self { 
            proxy_type: ProxyType::Socks4, 
            host: host.into(), 
            port, 
            username: user_id, 
            password: None 
        }
    }

    pub fn socks4a(host: String, port: u16, user_id: Option<String>) -> Self {
        Self { 
            proxy_type: ProxyType::Socks4a, 
            host: host.into(), 
            port, 
            username: user_id, 
            password: None 
        }
    }

    pub fn socks5(host: String, port: u16, username: Option<String>, password: Option<String>) -> Self {
        Self { 
            proxy_type: ProxyType::Socks5, 
            host: host.into(), 
            port, 
            username, 
            password 
        }
    }

    pub fn socks5h(host: String, port: u16, username: Option<String>, password: Option<String>) -> Self {
        Self { 
            proxy_type: ProxyType::Socks5h,
            host: host.into(), 
            port, 
            username, 
            password
        }
    }
}
