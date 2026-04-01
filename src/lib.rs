//! Official SDK for building powhttp extensions in Rust.
//!
//! An extension is started via [`run`], which provides an [`ExtensionHandle`] for
//! interacting with powhttp by registering context-menu items, overview
//! fields, connect handlers and querying session data.
//!
//! # Quick start
//!
//! ```rust,no_run
//! use powhttp_sdk::{run, Error, ExtensionHandle};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Error> {
//!     run(async |handle: ExtensionHandle| {
//!         // register handlers, query sessions, etc.
//!         Ok(())
//!     }).await
//! }
//! ```

mod error;
mod runtime;
mod serde_helpers;

/// TLS record-layer messages.
pub mod tls;
/// HTTP/2 frames.
pub mod http2;
/// WebSocket messages.
pub mod websocket;
/// Session entries and related data types (requests, responses, headers, timings).
pub mod sessions;
/// Proxy server connection handling and upstream proxy configuration.
pub mod proxy_server;
/// Context-menu items and submenus for single and multi-entry selections.
pub mod context_menu;
/// Overview fields and sections.
pub mod overview;
/// Small shared types used across multiple modules.
pub mod shared;

pub use runtime::run;
pub use error::Error;
pub use runtime::handle::ExtensionHandle;
pub use runtime::handlers::{SingleEntryContext, MultiEntryContext};
pub use overview::{OverviewField, OverviewSection};
pub use context_menu::{
    ContextMenuItemSingle,
    ContextMenuItemMulti,
    ContextMenuSubmenuSingle,
    ContextMenuSubmenuMulti
};
pub use sessions::Headers;