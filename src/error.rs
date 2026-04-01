/// Error type returned by extension handlers and SDK operations.
#[derive(Debug)]
pub enum Error {
    Handler(Box<dyn std::error::Error + Send + Sync>),
    Request(tokio_jrpc::ClientError),
    Runtime(tokio_jrpc::RuntimeError),
}

impl Error {
    /// Wraps any error type into [`Error::Handler`].
    pub fn new<E>(error: E) -> Self
    where E: Into<Box<dyn std::error::Error + Send + Sync>> {
        Self::Handler(error.into())
    }

    pub(crate) fn into_jrpc(self) -> tokio_jrpc::Error {
        let message = match self {
            Self::Handler(msg) => msg.to_string(),
            Self::Request(err) => err.to_string(),
            Self::Runtime(err) => err.to_string(),
        };
        tokio_jrpc::Error {
            code: tokio_jrpc::ErrorCode::InternalError,
            message,
            data: None,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Handler(e) => e.fmt(f),
            Self::Request(e) => e.fmt(f),
            Self::Runtime(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Handler(e) => Some(e.as_ref()),
            Self::Request(e) => Some(e),
            Self::Runtime(e) => Some(e),
        }
    }
}

impl From<tokio_jrpc::ClientError> for Error {
    fn from(error: tokio_jrpc::ClientError) -> Self {
        Self::Request(error)
    }
}

impl From<tokio_jrpc::RuntimeError> for Error {
    fn from(error: tokio_jrpc::RuntimeError) -> Self {
        Self::Runtime(error)
    }
}
