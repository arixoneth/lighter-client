use std::{
    sync::Arc,
    time::{Duration, Instant, SystemTime},
};

use tokio::sync::Mutex;

use super::errors::{Error, Result};
use crate::signer_client::SignerClient;

const AUTH_TTL_SECS: u64 = 9 * 60;

#[derive(Clone)]
pub(crate) struct AuthCache {
    inner: Arc<Mutex<AuthCacheState>>,
}

impl Default for AuthCache {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(AuthCacheState::default())),
        }
    }
}

#[derive(Default)]
struct AuthCacheState {
    token: Option<String>,
    expires_at: Option<Instant>,
}

impl AuthCacheState {
    fn is_valid(&self) -> bool {
        matches!(
            (&self.token, self.expires_at),
            (Some(_), Some(exp)) if Instant::now() < exp
        )
    }

    fn token(&self) -> Option<String> {
        self.token.clone()
    }

    fn clear(&mut self) {
        self.token = None;
        self.expires_at = None;
    }

    fn update(&mut self, token: String, expires_at: Instant) {
        self.token = Some(token);
        self.expires_at = Some(expires_at);
    }
}

impl AuthCache {
    pub(crate) async fn header(&self, signer: &SignerClient) -> Result<String> {
        if let Some(token) = self.current_token().await {
            return Ok(token);
        }

        let issued = signer.create_auth_token_with_expiry(None)?;
        let expiry = issued
            .expires_at
            .and_then(|deadline| deadline.duration_since(SystemTime::now()).ok())
            .map(|duration| Instant::now() + duration)
            .unwrap_or_else(|| Instant::now() + Duration::from_secs(AUTH_TTL_SECS));

        let mut guard = self.inner.lock().await;
        if guard.is_valid() {
            return guard.token().ok_or_else(Self::cache_empty_error);
        }

        guard.update(issued.token.clone(), expiry);
        Ok(issued.token)
    }

    pub(crate) async fn invalidate(&self) {
        let mut guard = self.inner.lock().await;
        guard.clear();
    }

    pub(crate) fn invalidate_blocking(&self) {
        let mut guard = self.inner.blocking_lock();
        guard.clear();
    }

    async fn current_token(&self) -> Option<String> {
        let guard = self.inner.lock().await;
        if guard.is_valid() {
            guard.token()
        } else {
            None
        }
    }

    fn cache_empty_error() -> Error {
        Error::Http {
            status: 500,
            body: "auth cache empty".to_string(),
        }
    }
}
