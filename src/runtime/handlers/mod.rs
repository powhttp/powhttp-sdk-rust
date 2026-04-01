use ulid::Ulid;

pub(crate) mod context_menu;
pub(crate) mod overview;
pub(crate) mod proxy_server;

#[derive(Debug)]
struct HandlerNotFound(String);

impl std::fmt::Display for HandlerNotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "no handler registered for '{}'", self.0)
    }
}

impl std::error::Error for HandlerNotFound {}

/// Context provided to handlers that operate on a single session entry.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SingleEntryContext {
    pub session_id: Ulid,
    pub entry_id: Ulid,
}

/// Context provided to handlers that operate on multiple session entries at once.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiEntryContext {
    pub session_id: Ulid,
    pub entry_ids: Vec<Ulid>,
}