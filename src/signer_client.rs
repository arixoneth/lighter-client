use std::{
    collections::HashMap,
    convert::TryFrom,
    path::Path,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use alloy_signer::{Error as AlloySignerError, SignerSync};
use alloy_signer_local::{LocalSignerError, PrivateKeySigner};
use serde::Deserialize;
use serde_json::{Map, Value};
use tokio::sync::Mutex;

use crate::{
    apis::{self, configuration, order_api, transaction_api},
    errors::{Result, SignerClientError},
    models,
    nonce_manager::{self, NonceManager, NonceManagerType},
    signer::SignerLibrary,
    timings, transactions,
};

const CODE_OK: i32 = 200;

const TX_TYPE_CHANGE_PUB_KEY: i32 = 8;
const TX_TYPE_CREATE_SUB_ACCOUNT: i32 = 9;
const TX_TYPE_CREATE_PUBLIC_POOL: i32 = 10;
const TX_TYPE_UPDATE_PUBLIC_POOL: i32 = 11;
const TX_TYPE_TRANSFER: i32 = 12;
const TX_TYPE_WITHDRAW: i32 = 13;
const TX_TYPE_CREATE_ORDER: i32 = 14;
const TX_TYPE_CANCEL_ORDER: i32 = 15;
const TX_TYPE_CANCEL_ALL_ORDERS: i32 = 16;
const TX_TYPE_MODIFY_ORDER: i32 = 17;
const TX_TYPE_MINT_SHARES: i32 = 18;
const TX_TYPE_BURN_SHARES: i32 = 19;
const TX_TYPE_UPDATE_LEVERAGE: i32 = 20;

const ORDER_TYPE_LIMIT: i32 = 0;
const ORDER_TYPE_MARKET: i32 = 1;
const ORDER_TYPE_STOP_LOSS: i32 = 2;
const ORDER_TYPE_STOP_LOSS_LIMIT: i32 = 3;
const ORDER_TYPE_TAKE_PROFIT: i32 = 4;
const ORDER_TYPE_TAKE_PROFIT_LIMIT: i32 = 5;

const ORDER_TIME_IN_FORCE_IMMEDIATE_OR_CANCEL: i32 = 0;
const ORDER_TIME_IN_FORCE_GOOD_TILL_TIME: i32 = 1;
const ORDER_TIME_IN_FORCE_POST_ONLY: i32 = 2;

const DEFAULT_28_DAY_ORDER_EXPIRY: i64 = -1;
const DEFAULT_IOC_EXPIRY: i64 = 0;
const DEFAULT_10_MIN_AUTH_EXPIRY: i64 = -1;
const NIL_TRIGGER_PRICE: i32 = 0;

const USDC_TICKER_SCALE: f64 = 1_000_000.0;
const MINUTE: i64 = 60;

pub struct SignerClient {
    signer: SignerLibrary,
    configuration: configuration::Configuration,
    start_api_key_index: i32,
    end_api_key_index: i32,
    account_index: i64,
    nonce_manager: Arc<Mutex<Box<dyn NonceManager>>>,
}

struct SigningContext {
    api_key_index: i32,
    nonce: i64,
    nonce_reserved: bool,
}

#[derive(Debug, Clone)]
pub struct AuthToken {
    pub token: String,
    pub expires_at: Option<SystemTime>,
}

#[derive(Debug, Clone)]
pub struct SignedPayload<T> {
    tx_type: i32,
    payload: String,
    parsed: T,
}

impl<T> SignedPayload<T> {
    fn new(tx_type: i32, payload: String, parsed: T) -> Self {
        Self {
            tx_type,
            payload,
            parsed,
        }
    }

    pub fn tx_type(&self) -> i32 {
        self.tx_type
    }

    pub fn payload(&self) -> &str {
        &self.payload
    }

    pub fn parsed(&self) -> &T {
        &self.parsed
    }

    pub fn into_parsed(self) -> T {
        self.parsed
    }

    pub fn into_parts(self) -> (BatchEntry, T) {
        let entry = BatchEntry {
            tx_type: self.tx_type,
            tx_info: self.payload,
        };
        (entry, self.parsed)
    }

    pub fn as_batch_entry(&self) -> BatchEntry {
        BatchEntry {
            tx_type: self.tx_type,
            tx_info: self.payload.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BatchEntry {
    pub tx_type: i32,
    pub tx_info: String,
}

impl BatchEntry {
    pub fn new(tx_type: i32, tx_info: impl Into<String>) -> Self {
        Self {
            tx_type,
            tx_info: tx_info.into(),
        }
    }

    pub fn tx_type(&self) -> i32 {
        self.tx_type
    }

    pub fn tx_info(&self) -> &str {
        &self.tx_info
    }
}

impl<T> From<&SignedPayload<T>> for BatchEntry {
    fn from(value: &SignedPayload<T>) -> Self {
        value.as_batch_entry()
    }
}

impl SignerClient {
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        url: impl Into<String>,
        private_key: impl AsRef<str>,
        api_key_index: i32,
        account_index: i64,
        max_api_key_index: Option<i32>,
        private_keys: Option<HashMap<i32, String>>,
        nonce_management_type: NonceManagerType,
        signer_library_path: Option<&Path>,
    ) -> Result<Self> {
        let url = url.into();
        let chain_id = if url.contains("mainnet") { 304 } else { 300 };

        let sanitized_initial_key = sanitize_private_key(private_key.as_ref());
        let end_api_key_index = max_api_key_index.unwrap_or(api_key_index);

        let api_key_map = prepare_api_key_map(
            api_key_index,
            end_api_key_index,
            &sanitized_initial_key,
            private_keys.unwrap_or_default(),
        )?;

        let signer = if let Some(path) = signer_library_path {
            SignerLibrary::load_from_path(path)?
        } else {
            SignerLibrary::load_default()?
        };

        let mut configuration = configuration::Configuration::default();
        configuration.base_path = url.clone();

        for idx in api_key_index..=end_api_key_index {
            let key = api_key_map.get(&idx).ok_or_else(|| {
                SignerClientError::Signer(format!("missing private key for api key {idx}"))
            })?;
            if let Some(err) = signer.create_client(&url, key, chain_id, idx, account_index)? {
                return Err(SignerClientError::Signer(err));
            }
        }

        let nonce_manager = nonce_manager::nonce_manager_factory(
            nonce_management_type,
            configuration.clone(),
            account_index,
            api_key_index,
            Some(end_api_key_index),
        )
        .await?;

        Ok(Self {
            signer,
            configuration,
            start_api_key_index: api_key_index,
            end_api_key_index,
            account_index,
            nonce_manager: Arc::new(Mutex::new(nonce_manager)),
        })
    }

    pub fn configuration(&self) -> configuration::Configuration {
        self.configuration.clone()
    }

    pub fn order_type_limit(&self) -> i32 {
        ORDER_TYPE_LIMIT
    }

    pub fn order_type_market(&self) -> i32 {
        ORDER_TYPE_MARKET
    }

    pub fn order_type_stop_loss(&self) -> i32 {
        ORDER_TYPE_STOP_LOSS
    }

    pub fn order_type_stop_loss_limit(&self) -> i32 {
        ORDER_TYPE_STOP_LOSS_LIMIT
    }

    pub fn order_type_take_profit(&self) -> i32 {
        ORDER_TYPE_TAKE_PROFIT
    }

    pub fn order_type_take_profit_limit(&self) -> i32 {
        ORDER_TYPE_TAKE_PROFIT_LIMIT
    }

    pub fn order_time_in_force_post_only(&self) -> i32 {
        ORDER_TIME_IN_FORCE_POST_ONLY
    }

    pub fn order_time_in_force_good_till_time(&self) -> i32 {
        ORDER_TIME_IN_FORCE_GOOD_TILL_TIME
    }

    pub fn order_time_in_force_immediate_or_cancel(&self) -> i32 {
        ORDER_TIME_IN_FORCE_IMMEDIATE_OR_CANCEL
    }

    pub async fn check_client(&self) -> Result<Option<String>> {
        self.signer
            .check_client(self.start_api_key_index, self.account_index)
    }

    pub fn default_ioc_expiry(&self) -> i64 {
        DEFAULT_IOC_EXPIRY
    }

    pub fn create_api_key(
        &self,
        seed: Option<&str>,
    ) -> Result<(Option<String>, Option<String>, Option<String>)> {
        self.signer.generate_api_key(seed)
    }

    pub fn create_auth_token_with_expiry(&self, deadline: Option<i64>) -> Result<AuthToken> {
        let deadline = match deadline {
            Some(value) if value != DEFAULT_10_MIN_AUTH_EXPIRY => value,
            _ => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64;
                now + 10 * MINUTE
            }
        };

        let (token, error) = self.signer.create_auth_token(deadline)?;
        let raw = parse_sign_output(token, error, "create_auth_token")?;
        if let Some(parsed) = parse_auth_token(&raw) {
            return Ok(parsed);
        }
        Ok(AuthToken {
            token: raw,
            expires_at: None,
        })
    }

    pub async fn change_api_key(
        &self,
        eth_private_key: &str,
        new_pubkey: &str,
        nonce: Option<i64>,
    ) -> Result<models::RespSendTx> {
        let (_, tx) = self
            .sign_change_pub_key_internal(eth_private_key, new_pubkey, nonce)
            .await?;
        Ok(tx)
    }

    async fn sign_change_pub_key_internal(
        &self,
        eth_private_key: &str,
        new_pubkey: &str,
        nonce: Option<i64>,
    ) -> Result<(String, models::RespSendTx)> {
        let context = self.prepare_context(None, nonce, false).await?;
        let (tx_info, error) = self.signer.sign_change_pub_key(new_pubkey, context.nonce)?;
        let tx_info = parse_sign_output(tx_info, error, "sign_change_pub_key")?;

        let mut payload: Map<String, Value> = serde_json::from_str(&tx_info)?;
        let message = payload
            .remove("MessageToSign")
            .ok_or(SignerClientError::InvalidResponse)?;
        let message = message
            .as_str()
            .ok_or_else(|| SignerClientError::InvalidResponse)?;

        let signature = sign_personal_message(eth_private_key, message)?;
        payload.insert("L1Sig".to_string(), Value::String(signature));

        let payload_string = serde_json::to_string(&payload)?;
        let response = self
            .submit_signed_tx(&context, TX_TYPE_CHANGE_PUB_KEY, &payload_string, None)
            .await?;
        Ok((payload_string, response))
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_order(
        &self,
        market_index: i32,
        client_order_index: i64,
        base_amount: i64,
        price: i32,
        is_ask: bool,
        order_type: i32,
        time_in_force: i32,
        reduce_only: bool,
        trigger_price: i32,
        order_expiry: i64,
        nonce: Option<i64>,
        api_key_index: Option<i32>,
    ) -> Result<(transactions::CreateOrder, models::RespSendTx)> {
        let context = self.prepare_context(api_key_index, nonce, true).await?;

        let signed = timings::time_block("sign_create_order", || {
            self.sign_create_order_with_context(
                market_index,
                client_order_index,
                base_amount,
                price,
                is_ask,
                order_type,
                time_in_force,
                reduce_only,
                trigger_price,
                order_expiry,
                &context,
            )
        })?;

        let payload = signed.payload().to_owned();
        let response = timings::time_async_block(
            "submit_signed_tx",
            self.submit_signed_tx(&context, signed.tx_type(), &payload, None),
        )
        .await?;
        Ok((signed.into_parsed(), response))
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn sign_create_order(
        &self,
        market_index: i32,
        client_order_index: i64,
        base_amount: i64,
        price: i32,
        is_ask: bool,
        order_type: i32,
        time_in_force: i32,
        reduce_only: bool,
        trigger_price: i32,
        order_expiry: i64,
        nonce: Option<i64>,
        api_key_index: Option<i32>,
    ) -> Result<SignedPayload<transactions::CreateOrder>> {
        let context = self.prepare_context(api_key_index, nonce, false).await?;
        self.sign_create_order_with_context(
            market_index,
            client_order_index,
            base_amount,
            price,
            is_ask,
            order_type,
            time_in_force,
            reduce_only,
            trigger_price,
            order_expiry,
            &context,
        )
    }

    pub async fn sign_market_order(
        &self,
        market_index: i32,
        client_order_index: i64,
        base_amount: i64,
        avg_execution_price: i32,
        is_ask: bool,
        reduce_only: bool,
        nonce: Option<i64>,
        api_key_index: Option<i32>,
    ) -> Result<SignedPayload<transactions::CreateOrder>> {
        self.sign_create_order(
            market_index,
            client_order_index,
            base_amount,
            avg_execution_price,
            is_ask,
            ORDER_TYPE_MARKET,
            ORDER_TIME_IN_FORCE_IMMEDIATE_OR_CANCEL,
            reduce_only,
            NIL_TRIGGER_PRICE,
            DEFAULT_IOC_EXPIRY,
            nonce,
            api_key_index,
        )
        .await
    }

    pub async fn sign_market_order_limited_slippage(
        &self,
        market_index: i32,
        client_order_index: i64,
        base_amount: i64,
        max_slippage: f64,
        is_ask: bool,
        reduce_only: bool,
        nonce: Option<i64>,
        api_key_index: Option<i32>,
        ideal_price: Option<i64>,
    ) -> Result<SignedPayload<transactions::CreateOrder>> {
        let acceptable_price = if let Some(price) = ideal_price {
            compute_slippage_price(price, max_slippage, is_ask)
        } else {
            let order_book = order_api::order_book_orders(&self.configuration, market_index, 1)
                .await
                .map_err(map_api_error)?;
            ensure_code_ok(order_book.code, order_book.message.as_deref())?;
            let top_order_price = if is_ask {
                order_book
                    .bids
                    .first()
                    .ok_or_else(|| SignerClientError::Signer("empty bid book".into()))?
                    .price
                    .clone()
            } else {
                order_book
                    .asks
                    .first()
                    .ok_or_else(|| SignerClientError::Signer("empty ask book".into()))?
                    .price
                    .clone()
            };
            let base_price = decimal_str_to_i64(&top_order_price)?;
            compute_slippage_price(base_price, max_slippage, is_ask)
        };

        self.sign_create_order(
            market_index,
            client_order_index,
            base_amount,
            acceptable_price as i32,
            is_ask,
            ORDER_TYPE_MARKET,
            ORDER_TIME_IN_FORCE_IMMEDIATE_OR_CANCEL,
            reduce_only,
            NIL_TRIGGER_PRICE,
            DEFAULT_IOC_EXPIRY,
            nonce,
            api_key_index,
        )
        .await
    }

    pub async fn sign_cancel_order(
        &self,
        market_index: i32,
        order_index: i64,
        nonce: Option<i64>,
        api_key_index: Option<i32>,
    ) -> Result<SignedPayload<transactions::CancelOrder>> {
        let context = self.prepare_context(api_key_index, nonce, false).await?;
        self.sign_cancel_order_with_context(market_index, order_index, &context)
    }

    pub async fn sign_withdraw(
        &self,
        usdc_amount: f64,
        nonce: Option<i64>,
        api_key_index: Option<i32>,
    ) -> Result<SignedPayload<transactions::Withdraw>> {
        let context = self.prepare_context(api_key_index, nonce, false).await?;
        self.sign_withdraw_with_context(usdc_amount, &context)
    }

    pub async fn send_signed_transaction(
        &self,
        tx_type: i32,
        tx_info: &str,
        price_protection: Option<bool>,
    ) -> Result<models::RespSendTx> {
        match transaction_api::send_tx(&self.configuration, tx_type, tx_info, price_protection)
            .await
        {
            Ok(response) => Ok(response),
            Err(err) => match classify_send_tx_error(err) {
                TxSendError::InvalidNonce(message) => Err(SignerClientError::Nonce(message)),
                TxSendError::Other(error) => Err(error),
            },
        }
    }

    pub async fn send_signed_payload<T>(
        &self,
        payload: &SignedPayload<T>,
        price_protection: Option<bool>,
    ) -> Result<models::RespSendTx> {
        self.send_signed_transaction(payload.tx_type(), payload.payload(), price_protection)
            .await
    }

    pub async fn send_tx_batch(&self, entries: &[BatchEntry]) -> Result<models::RespSendTxBatch> {
        if entries.is_empty() {
            return Err(SignerClientError::InvalidInput(
                "no transactions provided for batch submission".into(),
            ));
        }

        let tx_types: Vec<i32> = entries.iter().map(|entry| entry.tx_type).collect();
        let tx_infos: Vec<&str> = entries.iter().map(|entry| entry.tx_info.as_str()).collect();
        let types_json = serde_json::to_string(&tx_types)?;
        let infos_json = serde_json::to_string(&tx_infos)?;

        match transaction_api::send_tx_batch(&self.configuration, &types_json, &infos_json).await {
            Ok(response) => Ok(response),
            Err(err) => Err(map_api_error(err)),
        }
    }

    pub async fn send_signed_batch<T>(
        &self,
        payloads: &[SignedPayload<T>],
    ) -> Result<models::RespSendTxBatch> {
        let entries: Vec<BatchEntry> = payloads.iter().map(Into::into).collect();
        self.send_tx_batch(&entries).await
    }

    pub async fn next_nonce(&self) -> Result<(i32, i64)> {
        let mut manager = self.nonce_manager.lock().await;
        manager.next_nonce().await
    }

    pub async fn create_market_order(
        &self,
        market_index: i32,
        client_order_index: i64,
        base_amount: i64,
        avg_execution_price: i32,
        is_ask: bool,
        reduce_only: bool,
        nonce: Option<i64>,
        api_key_index: Option<i32>,
    ) -> Result<(transactions::CreateOrder, models::RespSendTx)> {
        self.create_order(
            market_index,
            client_order_index,
            base_amount,
            avg_execution_price,
            is_ask,
            ORDER_TYPE_MARKET,
            ORDER_TIME_IN_FORCE_IMMEDIATE_OR_CANCEL,
            reduce_only,
            NIL_TRIGGER_PRICE,
            DEFAULT_IOC_EXPIRY,
            nonce,
            api_key_index,
        )
        .await
    }

    pub async fn create_market_order_limited_slippage(
        &self,
        market_index: i32,
        client_order_index: i64,
        base_amount: i64,
        max_slippage: f64,
        is_ask: bool,
        reduce_only: bool,
        nonce: Option<i64>,
        api_key_index: Option<i32>,
        ideal_price: Option<i64>,
    ) -> Result<(transactions::CreateOrder, models::RespSendTx)> {
        let acceptable_price = if let Some(price) = ideal_price {
            compute_slippage_price(price, max_slippage, is_ask)
        } else {
            let order_book = order_api::order_book_orders(&self.configuration, market_index, 1)
                .await
                .map_err(map_api_error)?;
            ensure_code_ok(order_book.code, order_book.message.as_deref())?;
            let top_order_price = if is_ask {
                order_book
                    .bids
                    .first()
                    .ok_or_else(|| SignerClientError::Signer("empty bid book".into()))?
                    .price
                    .clone()
            } else {
                order_book
                    .asks
                    .first()
                    .ok_or_else(|| SignerClientError::Signer("empty ask book".into()))?
                    .price
                    .clone()
            };
            let base_price = decimal_str_to_i64(&top_order_price)?;
            compute_slippage_price(base_price, max_slippage, is_ask)
        };

        self.create_order(
            market_index,
            client_order_index,
            base_amount,
            acceptable_price as i32,
            is_ask,
            ORDER_TYPE_MARKET,
            ORDER_TIME_IN_FORCE_IMMEDIATE_OR_CANCEL,
            reduce_only,
            NIL_TRIGGER_PRICE,
            DEFAULT_IOC_EXPIRY,
            nonce,
            api_key_index,
        )
        .await
    }

    pub async fn create_market_order_if_slippage(
        &self,
        market_index: i32,
        client_order_index: i64,
        base_amount: i64,
        max_slippage: f64,
        is_ask: bool,
        reduce_only: bool,
        nonce: Option<i64>,
        api_key_index: Option<i32>,
        ideal_price: Option<i64>,
    ) -> Result<(transactions::CreateOrder, models::RespSendTx)> {
        let order_book = order_api::order_book_orders(&self.configuration, market_index, 100)
            .await
            .map_err(map_api_error)?;
        ensure_code_ok(order_book.code, order_book.message.as_deref())?;

        let ideal_price = if let Some(price) = ideal_price {
            price
        } else {
            let top_order = if is_ask {
                order_book
                    .bids
                    .first()
                    .ok_or_else(|| SignerClientError::Signer("order book is empty".into()))?
            } else {
                order_book
                    .asks
                    .first()
                    .ok_or_else(|| SignerClientError::Signer("order book is empty".into()))?
            };
            decimal_str_to_i64(&top_order.price)?
        };

        let orders = if is_ask {
            &order_book.bids
        } else {
            &order_book.asks
        };
        let mut matched_usd_amount: i128 = 0;
        let mut matched_size: i64 = 0;

        for order in orders {
            if matched_size >= base_amount {
                break;
            }
            let order_price = decimal_str_to_i64(&order.price)?;
            let remaining = decimal_str_to_i64(&order.remaining_base_amount)?;
            let usable_size = std::cmp::min(base_amount - matched_size, remaining);
            matched_usd_amount += (order_price as i128) * (usable_size as i128);
            matched_size += usable_size;
        }

        if matched_size < base_amount {
            return Err(SignerClientError::Signer(
                "Cannot be sure slippage will be acceptable due to the high size".into(),
            ));
        }

        let potential_execution_price = matched_usd_amount as f64 / matched_size as f64;
        let acceptable_execution_price =
            ideal_price as f64 * (1.0 + max_slippage * if is_ask { -1.0 } else { 1.0 });

        if (is_ask && potential_execution_price < acceptable_execution_price)
            || (!is_ask && potential_execution_price > acceptable_execution_price)
        {
            return Err(SignerClientError::Signer("Excessive slippage".into()));
        }

        self.create_order(
            market_index,
            client_order_index,
            base_amount,
            acceptable_execution_price.round() as i32,
            is_ask,
            ORDER_TYPE_MARKET,
            ORDER_TIME_IN_FORCE_IMMEDIATE_OR_CANCEL,
            reduce_only,
            NIL_TRIGGER_PRICE,
            DEFAULT_IOC_EXPIRY,
            nonce,
            api_key_index,
        )
        .await
    }

    pub async fn create_tp_order(
        &self,
        market_index: i32,
        client_order_index: i64,
        base_amount: i64,
        trigger_price: i32,
        price: i32,
        is_ask: bool,
        reduce_only: bool,
        nonce: Option<i64>,
        api_key_index: Option<i32>,
    ) -> Result<(transactions::CreateOrder, models::RespSendTx)> {
        self.create_order(
            market_index,
            client_order_index,
            base_amount,
            price,
            is_ask,
            ORDER_TYPE_TAKE_PROFIT,
            ORDER_TIME_IN_FORCE_IMMEDIATE_OR_CANCEL,
            reduce_only,
            trigger_price,
            DEFAULT_28_DAY_ORDER_EXPIRY,
            nonce,
            api_key_index,
        )
        .await
    }

    pub async fn create_tp_limit_order(
        &self,
        market_index: i32,
        client_order_index: i64,
        base_amount: i64,
        trigger_price: i32,
        price: i32,
        is_ask: bool,
        reduce_only: bool,
        nonce: Option<i64>,
        api_key_index: Option<i32>,
    ) -> Result<(transactions::CreateOrder, models::RespSendTx)> {
        self.create_order(
            market_index,
            client_order_index,
            base_amount,
            price,
            is_ask,
            ORDER_TYPE_TAKE_PROFIT_LIMIT,
            ORDER_TIME_IN_FORCE_GOOD_TILL_TIME,
            reduce_only,
            trigger_price,
            DEFAULT_28_DAY_ORDER_EXPIRY,
            nonce,
            api_key_index,
        )
        .await
    }

    pub async fn create_sl_order(
        &self,
        market_index: i32,
        client_order_index: i64,
        base_amount: i64,
        trigger_price: i32,
        price: i32,
        is_ask: bool,
        reduce_only: bool,
        nonce: Option<i64>,
        api_key_index: Option<i32>,
    ) -> Result<(transactions::CreateOrder, models::RespSendTx)> {
        self.create_order(
            market_index,
            client_order_index,
            base_amount,
            price,
            is_ask,
            ORDER_TYPE_STOP_LOSS,
            ORDER_TIME_IN_FORCE_IMMEDIATE_OR_CANCEL,
            reduce_only,
            trigger_price,
            DEFAULT_28_DAY_ORDER_EXPIRY,
            nonce,
            api_key_index,
        )
        .await
    }

    pub async fn create_sl_limit_order(
        &self,
        market_index: i32,
        client_order_index: i64,
        base_amount: i64,
        trigger_price: i32,
        price: i32,
        is_ask: bool,
        reduce_only: bool,
        nonce: Option<i64>,
        api_key_index: Option<i32>,
    ) -> Result<(transactions::CreateOrder, models::RespSendTx)> {
        self.create_order(
            market_index,
            client_order_index,
            base_amount,
            price,
            is_ask,
            ORDER_TYPE_STOP_LOSS_LIMIT,
            ORDER_TIME_IN_FORCE_GOOD_TILL_TIME,
            reduce_only,
            trigger_price,
            DEFAULT_28_DAY_ORDER_EXPIRY,
            nonce,
            api_key_index,
        )
        .await
    }

    pub async fn cancel_order(
        &self,
        market_index: i32,
        order_index: i64,
        nonce: Option<i64>,
        api_key_index: Option<i32>,
    ) -> Result<(transactions::CancelOrder, models::RespSendTx)> {
        let context = self.prepare_context(api_key_index, nonce, true).await?;
        let signed = self.sign_cancel_order_with_context(market_index, order_index, &context)?;
        let payload = signed.payload().to_owned();
        let response = self
            .submit_signed_tx(&context, signed.tx_type(), &payload, None)
            .await?;
        Ok((signed.into_parsed(), response))
    }

    pub async fn cancel_all_orders(
        &self,
        time_in_force: i32,
        time: i64,
        nonce: Option<i64>,
        api_key_index: Option<i32>,
    ) -> Result<(String, models::RespSendTx)> {
        let context = self.prepare_context(api_key_index, nonce, true).await?;
        let (tx_info, error) =
            self.signer
                .sign_cancel_all_orders(time_in_force, time, context.nonce)?;
        let tx_info = parse_sign_output(tx_info, error, "sign_cancel_all_orders")?;
        let response = self
            .submit_signed_tx(&context, TX_TYPE_CANCEL_ALL_ORDERS, &tx_info, None)
            .await?;
        Ok((tx_info, response))
    }

    pub async fn modify_order(
        &self,
        market_index: i32,
        order_index: i64,
        base_amount: i64,
        price: i64,
        trigger_price: i64,
        nonce: Option<i64>,
        api_key_index: Option<i32>,
    ) -> Result<(String, models::RespSendTx)> {
        let context = self.prepare_context(api_key_index, nonce, true).await?;
        let (tx_info, error) = self.signer.sign_modify_order(
            market_index,
            order_index,
            base_amount,
            price,
            trigger_price,
            context.nonce,
        )?;
        let tx_info = parse_sign_output(tx_info, error, "sign_modify_order")?;
        let response = self
            .submit_signed_tx(&context, TX_TYPE_MODIFY_ORDER, &tx_info, None)
            .await?;
        Ok((tx_info, response))
    }

    pub async fn withdraw(
        &self,
        usdc_amount: f64,
        nonce: Option<i64>,
        api_key_index: Option<i32>,
    ) -> Result<(transactions::Withdraw, models::RespSendTx)> {
        let context = self.prepare_context(api_key_index, nonce, true).await?;
        let signed = self.sign_withdraw_with_context(usdc_amount, &context)?;
        let payload = signed.payload().to_owned();
        let response = self
            .submit_signed_tx(&context, signed.tx_type(), &payload, None)
            .await?;
        Ok((signed.into_parsed(), response))
    }

    pub async fn create_sub_account(
        &self,
        nonce: Option<i64>,
    ) -> Result<(String, models::RespSendTx)> {
        let context = self.prepare_context(None, nonce, true).await?;
        let (tx_info, error) = self.signer.sign_create_sub_account(context.nonce)?;
        let tx_info = parse_sign_output(tx_info, error, "sign_create_sub_account")?;
        let response = self
            .submit_signed_tx(&context, TX_TYPE_CREATE_SUB_ACCOUNT, &tx_info, None)
            .await?;
        Ok((tx_info, response))
    }

    pub async fn transfer(
        &self,
        eth_private_key: &str,
        to_account_index: i64,
        usdc_amount: f64,
        fee: i64,
        memo: &str,
        nonce: Option<i64>,
        api_key_index: Option<i32>,
    ) -> Result<(String, models::RespSendTx)> {
        let context = self.prepare_context(api_key_index, nonce, true).await?;
        let scaled_amount = (usdc_amount * USDC_TICKER_SCALE) as i64;
        let (tx_info, error) =
            self.signer
                .sign_transfer(to_account_index, scaled_amount, fee, memo, context.nonce)?;
        let tx_info = parse_sign_output(tx_info, error, "sign_transfer")?;

        let mut payload: Map<String, Value> = serde_json::from_str(&tx_info)?;
        let msg = payload
            .remove("MessageToSign")
            .ok_or(SignerClientError::InvalidResponse)?;
        let message = msg.as_str().ok_or(SignerClientError::InvalidResponse)?;
        let signature = sign_personal_message(eth_private_key, message)?;
        payload.insert("L1Sig".into(), Value::String(signature));

        let payload_string = serde_json::to_string(&payload)?;
        let response = self
            .submit_signed_tx(&context, TX_TYPE_TRANSFER, &payload_string, None)
            .await?;
        Ok((payload_string, response))
    }

    pub async fn create_public_pool(
        &self,
        operator_fee: i64,
        initial_total_shares: i64,
        min_operator_share_rate: i64,
        nonce: Option<i64>,
        api_key_index: Option<i32>,
    ) -> Result<(String, models::RespSendTx)> {
        let context = self.prepare_context(api_key_index, nonce, true).await?;
        let (tx_info, error) = self.signer.sign_create_public_pool(
            operator_fee,
            initial_total_shares,
            min_operator_share_rate,
            context.nonce,
        )?;
        let tx_info = parse_sign_output(tx_info, error, "sign_create_public_pool")?;
        let response = self
            .submit_signed_tx(&context, TX_TYPE_CREATE_PUBLIC_POOL, &tx_info, None)
            .await?;
        Ok((tx_info, response))
    }

    pub async fn update_public_pool(
        &self,
        public_pool_index: i64,
        status: i32,
        operator_fee: i64,
        min_operator_share_rate: i64,
        nonce: Option<i64>,
        api_key_index: Option<i32>,
    ) -> Result<(String, models::RespSendTx)> {
        let context = self.prepare_context(api_key_index, nonce, true).await?;
        let (tx_info, error) = self.signer.sign_update_public_pool(
            public_pool_index,
            status,
            operator_fee,
            min_operator_share_rate,
            context.nonce,
        )?;
        let tx_info = parse_sign_output(tx_info, error, "sign_update_public_pool")?;
        let response = self
            .submit_signed_tx(&context, TX_TYPE_UPDATE_PUBLIC_POOL, &tx_info, None)
            .await?;
        Ok((tx_info, response))
    }

    pub async fn mint_shares(
        &self,
        public_pool_index: i64,
        share_amount: i64,
        nonce: Option<i64>,
        api_key_index: Option<i32>,
    ) -> Result<(String, models::RespSendTx)> {
        let context = self.prepare_context(api_key_index, nonce, true).await?;
        let (tx_info, error) =
            self.signer
                .sign_mint_shares(public_pool_index, share_amount, context.nonce)?;
        let tx_info = parse_sign_output(tx_info, error, "sign_mint_shares")?;
        let response = self
            .submit_signed_tx(&context, TX_TYPE_MINT_SHARES, &tx_info, None)
            .await?;
        Ok((tx_info, response))
    }

    pub async fn burn_shares(
        &self,
        public_pool_index: i64,
        share_amount: i64,
        nonce: Option<i64>,
        api_key_index: Option<i32>,
    ) -> Result<(String, models::RespSendTx)> {
        let context = self.prepare_context(api_key_index, nonce, true).await?;
        let (tx_info, error) =
            self.signer
                .sign_burn_shares(public_pool_index, share_amount, context.nonce)?;
        let tx_info = parse_sign_output(tx_info, error, "sign_burn_shares")?;
        let response = self
            .submit_signed_tx(&context, TX_TYPE_BURN_SHARES, &tx_info, None)
            .await?;
        Ok((tx_info, response))
    }

    pub async fn update_leverage(
        &self,
        market_index: i32,
        margin_mode: i32,
        leverage: i32,
        nonce: Option<i64>,
        api_key_index: Option<i32>,
    ) -> Result<(String, models::RespSendTx)> {
        let context = self.prepare_context(api_key_index, nonce, true).await?;
        let fraction = (10_000f64 / leverage as f64).round() as i32;
        let (tx_info, error) =
            self.signer
                .sign_update_leverage(market_index, fraction, margin_mode, context.nonce)?;
        let tx_info = parse_sign_output(tx_info, error, "sign_update_leverage")?;
        let response = self
            .submit_signed_tx(&context, TX_TYPE_UPDATE_LEVERAGE, &tx_info, None)
            .await?;
        Ok((tx_info, response))
    }

    #[allow(clippy::too_many_arguments)]
    fn sign_create_order_with_context(
        &self,
        market_index: i32,
        client_order_index: i64,
        base_amount: i64,
        price: i32,
        is_ask: bool,
        order_type: i32,
        time_in_force: i32,
        reduce_only: bool,
        trigger_price: i32,
        order_expiry: i64,
        context: &SigningContext,
    ) -> Result<SignedPayload<transactions::CreateOrder>> {
        let (tx_info, error) = self.signer.sign_create_order(
            market_index,
            client_order_index,
            base_amount,
            price,
            is_ask,
            order_type,
            time_in_force,
            reduce_only,
            trigger_price,
            order_expiry,
            context.nonce,
        )?;
        let tx_info = parse_sign_output(tx_info, error, "sign_create_order")?;
        let parsed = transactions::CreateOrder::from_json_str(&tx_info)?;
        Ok(SignedPayload::new(TX_TYPE_CREATE_ORDER, tx_info, parsed))
    }

    fn sign_cancel_order_with_context(
        &self,
        market_index: i32,
        order_index: i64,
        context: &SigningContext,
    ) -> Result<SignedPayload<transactions::CancelOrder>> {
        let (tx_info, error) =
            self.signer
                .sign_cancel_order(market_index, order_index, context.nonce)?;
        let tx_info = parse_sign_output(tx_info, error, "sign_cancel_order")?;
        let parsed = transactions::CancelOrder::from_json_str(&tx_info)?;
        Ok(SignedPayload::new(TX_TYPE_CANCEL_ORDER, tx_info, parsed))
    }

    fn sign_withdraw_with_context(
        &self,
        usdc_amount: f64,
        context: &SigningContext,
    ) -> Result<SignedPayload<transactions::Withdraw>> {
        let scaled_amount = (usdc_amount * USDC_TICKER_SCALE) as i64;
        let (tx_info, error) = self.signer.sign_withdraw(scaled_amount, context.nonce)?;
        let tx_info = parse_sign_output(tx_info, error, "sign_withdraw")?;
        let parsed = transactions::Withdraw::from_json_str(&tx_info)?;
        Ok(SignedPayload::new(TX_TYPE_WITHDRAW, tx_info, parsed))
    }

    async fn prepare_context(
        &self,
        api_key_index: Option<i32>,
        nonce: Option<i64>,
        reserve_nonce: bool,
    ) -> Result<SigningContext> {
        let mut api_key = api_key_index.unwrap_or(-1);
        let mut nonce_value = nonce.unwrap_or(-1);
        let mut nonce_reserved = false;

        if reserve_nonce && api_key == -1 && nonce_value == -1 {
            let (key, value) = {
                let mut manager = self.nonce_manager.lock().await;
                manager.next_nonce().await?
            };
            api_key = key;
            nonce_value = value;
            nonce_reserved = true;
        } else if api_key == -1 {
            api_key = self.start_api_key_index;
        }

        if api_key < self.start_api_key_index || api_key > self.end_api_key_index {
            return Err(SignerClientError::Signer(format!(
                "api key index {api_key} out of range"
            )));
        }

        if let Some(err) = self.signer.switch_api_key(api_key)? {
            return Err(SignerClientError::Signer(err));
        }

        Ok(SigningContext {
            api_key_index: api_key,
            nonce: nonce_value,
            nonce_reserved,
        })
    }

    async fn submit_signed_tx(
        &self,
        context: &SigningContext,
        tx_type: i32,
        tx_info: &str,
        price_protection: Option<bool>,
    ) -> Result<models::RespSendTx> {
        match transaction_api::send_tx(&self.configuration, tx_type, tx_info, price_protection)
            .await
        {
            Ok(response) => {
                if response.code != CODE_OK && context.nonce_reserved {
                    self.acknowledge_failure(context.api_key_index).await;
                }
                Ok(response)
            }
            Err(err) => {
                if context.nonce_reserved {
                    self.acknowledge_failure(context.api_key_index).await;
                }
                match classify_send_tx_error(err) {
                    TxSendError::InvalidNonce(message) => {
                        if context.nonce_reserved {
                            self.hard_refresh_nonce(context.api_key_index).await?;
                        }
                        Err(SignerClientError::Nonce(message))
                    }
                    TxSendError::Other(error) => Err(error),
                }
            }
        }
    }

    async fn acknowledge_failure(&self, api_key_index: i32) {
        let mut manager = self.nonce_manager.lock().await;
        manager.acknowledge_failure(api_key_index);
    }

    async fn hard_refresh_nonce(&self, api_key_index: i32) -> Result<()> {
        let mut manager = self.nonce_manager.lock().await;
        manager.hard_refresh_nonce(api_key_index).await
    }
}

pub async fn create_api_key(
    seed: Option<&str>,
) -> Result<(Option<String>, Option<String>, Option<String>)> {
    let signer = SignerLibrary::load_default()?;
    signer.generate_api_key(seed)
}

enum TxSendError {
    InvalidNonce(String),
    Other(SignerClientError),
}

fn classify_send_tx_error(err: apis::Error<transaction_api::SendTxError>) -> TxSendError {
    match err {
        apis::Error::Reqwest(e) => TxSendError::Other(SignerClientError::Reqwest(e)),
        apis::Error::Serde(e) => TxSendError::Other(SignerClientError::Json(e)),
        apis::Error::Io(e) => TxSendError::Other(SignerClientError::Io(e)),
        apis::Error::ResponseError(mut response) => {
            let message = response.entity.as_ref().and_then(extract_send_tx_message);
            if let Some(msg) = message.clone() {
                if msg.to_ascii_lowercase().contains("invalid nonce") {
                    return TxSendError::InvalidNonce(msg);
                }
            }

            let entity_string = response
                .entity
                .take()
                .and_then(|entity| serde_json::to_string(&entity).ok());
            TxSendError::Other(SignerClientError::with_http_status(
                response.status,
                entity_string,
                &response.content,
            ))
        }
    }
}

fn extract_send_tx_message(error: &transaction_api::SendTxError) -> Option<String> {
    match error {
        transaction_api::SendTxError::Status400(result) => result.message.clone(),
        transaction_api::SendTxError::UnknownValue(value) => Some(value.to_string()),
    }
}

fn sign_personal_message(private_key: &str, message: &str) -> Result<String> {
    let key = sanitize_private_key(private_key);
    let signer: PrivateKeySigner = key
        .parse()
        .map_err(|err: LocalSignerError| SignerClientError::Signing(err.to_string()))?;
    let signature = signer
        .sign_message_sync(message.as_bytes())
        .map_err(|err: AlloySignerError| SignerClientError::Signing(err.to_string()))?;
    Ok(signature.to_string())
}

#[derive(Debug, Deserialize)]
struct RawAuthToken {
    #[serde(alias = "token", alias = "authToken", alias = "auth_token")]
    token: Option<String>,
    #[serde(
        alias = "expires_at",
        alias = "expiry",
        alias = "expiresAt",
        alias = "deadline"
    )]
    expires_at: Option<i64>,
}

fn parse_auth_token(raw: &str) -> Option<AuthToken> {
    let payload: RawAuthToken = serde_json::from_str(raw).ok()?;
    let token = payload.token?;
    let expires_at = payload.expires_at.and_then(unix_timestamp_to_system_time);
    Some(AuthToken { token, expires_at })
}

fn unix_timestamp_to_system_time(value: i64) -> Option<SystemTime> {
    if value <= 0 {
        return None;
    }

    let (secs, nanos) = if value > 1_000_000_000_000 {
        let secs = value / 1_000;
        let ms = value % 1_000;
        (secs, ms * 1_000_000)
    } else {
        (value, 0)
    };

    let secs = u64::try_from(secs).ok()?;
    let nanos = u64::try_from(nanos).ok()?;
    let base = UNIX_EPOCH.checked_add(Duration::from_secs(secs))?;
    base.checked_add(Duration::from_nanos(nanos))
}

fn parse_sign_output(
    value: Option<String>,
    error: Option<String>,
    context: &str,
) -> Result<String> {
    if let Some(err) = error {
        return Err(SignerClientError::Signer(format!("{context}: {err}")));
    }
    value.ok_or(SignerClientError::InvalidResponse)
}

fn compute_slippage_price(base_price: i64, max_slippage: f64, is_ask: bool) -> i64 {
    let factor = if is_ask {
        1.0 - max_slippage
    } else {
        1.0 + max_slippage
    };
    (base_price as f64 * factor).round() as i64
}

fn decimal_str_to_i64(value: &str) -> Result<i64> {
    let filtered: String = value.chars().filter(|c| *c != '.').collect();
    filtered
        .parse::<i64>()
        .map_err(|err| SignerClientError::Signer(format!("invalid numeric value {value}: {err}")))
}

pub(crate) fn ensure_code_ok(code: i32, message: Option<&str>) -> Result<()> {
    if code == CODE_OK {
        return Ok(());
    }
    Err(SignerClientError::Http(
        message.unwrap_or("unexpected response code").to_string(),
    ))
}

fn sanitize_private_key(value: &str) -> String {
    value.trim_start_matches("0x").to_string()
}

fn prepare_api_key_map(
    start_api_key: i32,
    end_api_key: i32,
    initial_private_key: &str,
    mut provided: HashMap<i32, String>,
) -> Result<HashMap<i32, String>> {
    for value in provided.values_mut() {
        *value = sanitize_private_key(value);
    }

    let expected_range = (end_api_key - start_api_key) as usize;
    if provided.len() == expected_range + 1 {
        let existing = provided
            .get(&start_api_key)
            .ok_or_else(|| SignerClientError::Signer("inconsistent private keys".into()))?;
        if !are_keys_equal(existing, initial_private_key) {
            return Err(SignerClientError::Signer(
                "inconsistent private keys".into(),
            ));
        }
    } else if provided.len() == expected_range {
        provided.insert(start_api_key, initial_private_key.to_string());
    } else if provided.is_empty() {
        provided.insert(start_api_key, initial_private_key.to_string());
    } else {
        return Err(SignerClientError::Signer(
            "unexpected number of private keys provided".into(),
        ));
    }

    for idx in start_api_key..=end_api_key {
        if !provided.contains_key(&idx) {
            return Err(SignerClientError::Signer(format!(
                "missing private key for api key {idx}"
            )));
        }
    }

    Ok(provided)
}

fn are_keys_equal(a: &str, b: &str) -> bool {
    sanitize_private_key(a) == sanitize_private_key(b)
}

pub(crate) fn map_api_error<E>(err: apis::Error<E>) -> SignerClientError
where
    E: serde::Serialize + std::fmt::Debug,
{
    match err {
        apis::Error::Reqwest(e) => SignerClientError::Reqwest(e),
        apis::Error::Serde(e) => SignerClientError::Json(e),
        apis::Error::Io(e) => SignerClientError::Io(e),
        apis::Error::ResponseError(mut response) => {
            let entity = response
                .entity
                .take()
                .and_then(|entity| serde_json::to_string(&entity).ok());
            SignerClientError::with_http_status(response.status, entity, &response.content)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::sanitize_private_key;

    #[test]
    fn test_sanitize_private_key() {
        assert_eq!(sanitize_private_key("0xabc"), "abc");
        assert_eq!(sanitize_private_key("abc"), "abc");
    }
}
