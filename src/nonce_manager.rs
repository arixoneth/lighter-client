use std::collections::HashMap;

use async_trait::async_trait;

use crate::{
    apis,
    apis::{configuration::Configuration, transaction_api},
    errors::{Result, SignerClientError},
    models,
};

#[derive(Debug, Clone, Copy)]
pub enum NonceManagerType {
    Optimistic,
    Api,
}

#[async_trait]
pub trait NonceManager: Send + Sync {
    async fn next_nonce(&mut self) -> Result<(i32, i64)>;
    async fn hard_refresh_nonce(&mut self, api_key_index: i32) -> Result<()>;
    fn acknowledge_failure(&mut self, api_key_index: i32);
}

pub struct OptimisticNonceManager {
    state: NonceState,
}

pub struct ApiNonceManager {
    state: NonceState,
}

struct NonceState {
    configuration: Configuration,
    account_index: i64,
    start_api_key: i32,
    end_api_key: i32,
    current_api_key: i32,
    nonce: HashMap<i32, i64>,
}

impl NonceState {
    async fn new(
        configuration: Configuration,
        account_index: i64,
        start_api_key: i32,
        end_api_key: i32,
    ) -> Result<Self> {
        if start_api_key > end_api_key || start_api_key < 0 || end_api_key >= 255 {
            return Err(SignerClientError::Nonce(format!(
                "invalid api key range start={} end={}",
                start_api_key, end_api_key
            )));
        }

        let mut nonce = HashMap::new();
        for api_key_index in start_api_key..=end_api_key {
            let next_nonce = fetch_nonce(&configuration, account_index, api_key_index).await?;
            nonce.insert(api_key_index, next_nonce - 1);
        }

        Ok(Self {
            configuration,
            account_index,
            start_api_key,
            end_api_key,
            current_api_key: end_api_key,
            nonce,
        })
    }

    fn increment_current_api_key(&mut self) -> i32 {
        self.current_api_key += 1;
        if self.current_api_key > self.end_api_key {
            self.current_api_key = self.start_api_key;
        }
        self.current_api_key
    }

    async fn hard_refresh_nonce(&mut self, api_key_index: i32) -> Result<()> {
        let nonce_value =
            fetch_nonce(&self.configuration, self.account_index, api_key_index).await?;
        self.nonce.insert(api_key_index, nonce_value - 1);
        Ok(())
    }

    fn acknowledge_failure(&mut self, api_key_index: i32) {
        if let Some(entry) = self.nonce.get_mut(&api_key_index) {
            *entry -= 1;
        }
    }
}

#[async_trait]
impl NonceManager for OptimisticNonceManager {
    async fn next_nonce(&mut self) -> Result<(i32, i64)> {
        let api_key = self.state.increment_current_api_key();
        let entry = self.state.nonce.get_mut(&api_key).ok_or_else(|| {
            SignerClientError::Nonce(format!("missing nonce for api key {}", api_key))
        })?;
        *entry += 1;
        Ok((api_key, *entry))
    }

    async fn hard_refresh_nonce(&mut self, api_key_index: i32) -> Result<()> {
        self.state.hard_refresh_nonce(api_key_index).await
    }

    fn acknowledge_failure(&mut self, api_key_index: i32) {
        self.state.acknowledge_failure(api_key_index);
    }
}

#[async_trait]
impl NonceManager for ApiNonceManager {
    async fn next_nonce(&mut self) -> Result<(i32, i64)> {
        let api_key = self.state.increment_current_api_key();
        let nonce_value =
            fetch_nonce(&self.state.configuration, self.state.account_index, api_key).await?;
        self.state.nonce.insert(api_key, nonce_value);
        Ok((api_key, nonce_value))
    }

    async fn hard_refresh_nonce(&mut self, api_key_index: i32) -> Result<()> {
        self.state.hard_refresh_nonce(api_key_index).await
    }

    fn acknowledge_failure(&mut self, api_key_index: i32) {
        self.state.acknowledge_failure(api_key_index);
    }
}

pub async fn nonce_manager_factory(
    manager_type: NonceManagerType,
    configuration: Configuration,
    account_index: i64,
    start_api_key: i32,
    end_api_key: Option<i32>,
) -> Result<Box<dyn NonceManager>> {
    let end_api_key = end_api_key.unwrap_or(start_api_key);

    match manager_type {
        NonceManagerType::Optimistic => {
            let state =
                NonceState::new(configuration, account_index, start_api_key, end_api_key).await?;
            Ok(Box::new(OptimisticNonceManager { state }))
        }
        NonceManagerType::Api => {
            let state =
                NonceState::new(configuration, account_index, start_api_key, end_api_key).await?;
            Ok(Box::new(ApiNonceManager { state }))
        }
    }
}

async fn fetch_nonce(
    configuration: &Configuration,
    account_index: i64,
    api_key_index: i32,
) -> Result<i64> {
    match transaction_api::next_nonce(configuration, account_index, api_key_index).await {
        Ok(models::NextNonce {
            code,
            nonce,
            message,
        }) => {
            if code != 200 {
                return Err(SignerClientError::Nonce(
                    message.unwrap_or_else(|| format!("unexpected code {}", code)),
                ));
            }
            Ok(nonce)
        }
        Err(err) => Err(map_api_error(err)),
    }
}

fn map_api_error<E>(err: apis::Error<E>) -> SignerClientError
where
    E: serde::Serialize + std::fmt::Debug,
{
    match err {
        apis::Error::Reqwest(e) => SignerClientError::Reqwest(e),
        apis::Error::Serde(e) => SignerClientError::Json(e),
        apis::Error::Io(e) => SignerClientError::Io(e),
        apis::Error::ResponseError(response) => {
            let entity = response.entity.and_then(|e| serde_json::to_string(&e).ok());
            SignerClientError::with_http_status(response.status, entity, &response.content)
        }
    }
}
