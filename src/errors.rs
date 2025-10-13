use std::fmt;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, SignerClientError>;

#[derive(Debug, Error)]
pub enum SignerClientError {
    #[error("signer library error: {0}")]
    Signer(String),
    #[error("unsupported platform {0}")]
    UnsupportedPlatform(String),
    #[error("nonce error: {0}")]
    Nonce(String),
    #[error("http error: {0}")]
    Http(String),
    #[error("signing error: {0}")]
    Signing(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("invalid response from signer library")]
    InvalidResponse,
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Utf8(#[from] std::str::Utf8Error),
    #[error(transparent)]
    FromUtf8(#[from] std::string::FromUtf8Error),
    #[error(transparent)]
    Library(#[from] libloading::Error),
    #[error(transparent)]
    CString(#[from] std::ffi::NulError),
}

impl SignerClientError {
    pub fn with_http_status(
        status: reqwest::StatusCode,
        entity: Option<String>,
        body: &str,
    ) -> Self {
        let mut msg = format!("status code {}", status);
        if let Some(entity) = entity {
            msg.push_str(": ");
            msg.push_str(&entity);
        } else if !body.is_empty() {
            msg.push_str(": ");
            msg.push_str(body);
        }

        SignerClientError::Http(msg)
    }
}

pub type WsResult<T> = std::result::Result<T, WsClientError>;

#[derive(Debug, Error)]
pub enum WsClientError {
    #[error("no subscriptions provided")]
    EmptySubscriptions,
    #[error("invalid websocket channel: {0}")]
    InvalidChannel(String),
    #[error("invalid websocket message: {0}")]
    InvalidMessage(String),
    #[error("message missing type field")]
    MissingMessageType,
    #[error("unsupported websocket scheme {0}")]
    UnsupportedScheme(String),
    #[error(transparent)]
    Url(#[from] url::ParseError),
    #[error(transparent)]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
