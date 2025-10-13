use serde::Deserialize;

use crate::errors::{SignerClientError, WsClientError};

/// Result type used by [`LighterClient`](super::LighterClient).
pub type Result<T> = std::result::Result<T, Error>;

/// Errors returned by the high level client API.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Errors bubbled up from the signing client.
    #[error("signer error: {0}")]
    Signer(#[from] SignerClientError),
    /// Errors that originate from the websocket client helpers.
    #[error("ws error: {0}")]
    Ws(#[from] WsClientError),
    /// Configuration validation failure raised by a builder.
    #[error("invalid config: {field}: {why}")]
    InvalidConfig {
        field: &'static str,
        why: &'static str,
    },
    /// Attempted to call an authenticated method without configuring an account.
    #[error("unauthenticated")]
    NotAuthenticated,
    /// Requests were rate limited by the server.
    #[error("rate limited: retry after {retry_after:?}s")]
    RateLimited { retry_after: Option<u64> },
    /// Structured server error response.
    #[error("server error {status}: {message}")]
    Server {
        status: u16,
        message: String,
        code: Option<String>,
    },
    /// Raw HTTP error when no structured error could be parsed.
    #[error("http {status}: {body}")]
    Http { status: u16, body: String },
}

#[derive(Debug, Deserialize)]
pub(crate) struct ServerErr {
    pub(crate) message: String,
    #[serde(default)]
    pub(crate) code: Option<String>,
    #[serde(default)]
    pub(crate) retry_after: Option<u64>,
}
