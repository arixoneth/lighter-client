use std::fmt;

use super::{
    auth::AuthCache,
    errors::{Error, Result, ServerErr},
    params::{By, SortDir, TradeSort},
};
use crate::{
    apis::{
        self, account_api, block_api, bridge_api, candlestick_api, configuration, funding_api,
        info_api, notification_api, order_api, transaction_api,
    },
    models,
    types::{AccountId, ApiKeyIndex, MarketId},
};

/// Thin wrapper around the generated REST configuration and endpoints.
pub struct RestClient {
    configuration: configuration::Configuration,
    auth: AuthCache,
}

impl RestClient {
    pub(crate) fn new(base_path: impl Into<String>, auth: AuthCache) -> Self {
        let mut configuration = configuration::Configuration::default();
        configuration.base_path = base_path.into();
        Self {
            configuration,
            auth,
        }
    }

    pub(crate) fn configuration(&self) -> configuration::Configuration {
        self.configuration.clone()
    }

    pub(crate) fn configuration_ref(&self) -> &configuration::Configuration {
        &self.configuration
    }

    pub(crate) fn set_configuration(&mut self, configuration: configuration::Configuration) {
        self.configuration = configuration;
    }

    pub(crate) fn base_path(&self) -> &str {
        &self.configuration.base_path
    }

    pub(crate) async fn order_book(
        &self,
        market: MarketId,
        limit: i64,
    ) -> Result<models::OrderBookOrders> {
        order_api::order_book_orders(self.configuration_ref(), market.into(), limit)
            .await
            .map_err(|err| self.map_rest_error(err))
    }

    pub(crate) async fn account_details(
        &self,
        account_value: &str,
    ) -> Result<models::DetailedAccounts> {
        account_api::account(self.configuration_ref(), "index", account_value)
            .await
            .map_err(|err| self.map_rest_error(err))
    }

    pub(crate) async fn account_limits(
        &self,
        account_index: i64,
        auth: &str,
    ) -> Result<models::AccountLimits> {
        account_api::account_limits(
            self.configuration_ref(),
            account_index,
            Some(auth),
            Some(auth),
        )
        .await
        .map_err(|err| self.map_rest_error(err))
    }

    pub(crate) async fn account_metadata(
        &self,
        account_value: &str,
        auth: &str,
    ) -> Result<models::AccountMetadatas> {
        account_api::account_metadata(
            self.configuration_ref(),
            "index",
            account_value,
            Some(auth),
            Some(auth),
        )
        .await
        .map_err(|err| self.map_rest_error(err))
    }

    pub(crate) async fn api_keys(
        &self,
        account_index: i64,
        api_key_index: Option<ApiKeyIndex>,
    ) -> Result<models::AccountApiKeys> {
        account_api::apikeys(
            self.configuration_ref(),
            account_index,
            api_key_index.map(Into::into),
        )
        .await
        .map_err(|err| self.map_rest_error(err))
    }

    pub(crate) async fn change_account_tier(
        &self,
        account_index: i64,
        tier: &str,
        auth: &str,
    ) -> Result<models::RespChangeAccountTier> {
        account_api::change_account_tier(
            self.configuration_ref(),
            account_index,
            tier,
            Some(auth),
            Some(auth),
        )
        .await
        .map_err(|err| self.map_rest_error(err))
    }

    pub(crate) async fn l1_metadata(
        &self,
        l1_address: &str,
        auth: &str,
    ) -> Result<models::L1Metadata> {
        account_api::l1_metadata(self.configuration_ref(), l1_address, Some(auth), Some(auth))
            .await
            .map_err(|err| self.map_rest_error(err))
    }

    pub(crate) async fn liquidations(
        &self,
        account_index: i64,
        limit: i64,
        market: Option<MarketId>,
        cursor: Option<&str>,
        auth: &str,
    ) -> Result<models::LiquidationInfos> {
        account_api::liquidations(
            self.configuration_ref(),
            account_index,
            limit,
            Some(auth),
            Some(auth),
            market.map(Into::into),
            cursor,
        )
        .await
        .map_err(|err| self.map_rest_error(err))
    }

    pub(crate) async fn account_pnl(
        &self,
        account_value: &str,
        resolution: &str,
        start_timestamp: i64,
        end_timestamp: i64,
        count_back: i64,
        ignore_transfers: Option<bool>,
        auth: &str,
    ) -> Result<models::AccountPnL> {
        account_api::pnl(
            self.configuration_ref(),
            "index",
            account_value,
            resolution,
            start_timestamp,
            end_timestamp,
            count_back,
            Some(auth),
            Some(auth),
            ignore_transfers,
        )
        .await
        .map_err(|err| self.map_rest_error(err))
    }

    pub(crate) async fn position_funding(
        &self,
        account_index: i64,
        limit: i64,
        market: Option<MarketId>,
        cursor: Option<&str>,
        side: Option<&str>,
        auth: &str,
    ) -> Result<models::PositionFundings> {
        account_api::position_funding(
            self.configuration_ref(),
            account_index,
            limit,
            Some(auth),
            Some(auth),
            market.map(Into::into),
            cursor,
            side,
        )
        .await
        .map_err(|err| self.map_rest_error(err))
    }

    pub(crate) async fn public_pools(
        &self,
        index: i64,
        limit: i64,
        filter: Option<&str>,
        account_index: i64,
        auth: &str,
    ) -> Result<models::PublicPools> {
        account_api::public_pools(
            self.configuration_ref(),
            index,
            limit,
            Some(auth),
            Some(auth),
            filter,
            Some(account_index),
        )
        .await
        .map_err(|err| self.map_rest_error(err))
    }

    pub(crate) async fn public_pools_metadata(
        &self,
        index: i64,
        limit: i64,
        filter: Option<&str>,
        account_index: i64,
        auth: &str,
    ) -> Result<models::RespPublicPoolsMetadata> {
        account_api::public_pools_metadata(
            self.configuration_ref(),
            index,
            limit,
            Some(auth),
            Some(auth),
            filter,
            Some(account_index),
        )
        .await
        .map_err(|err| self.map_rest_error(err))
    }

    pub(crate) async fn accounts_by_l1_address(
        &self,
        l1_address: &str,
    ) -> Result<models::SubAccounts> {
        account_api::accounts_by_l1_address(self.configuration_ref(), l1_address)
            .await
            .map_err(|err| self.map_rest_error(err))
    }

    pub(crate) async fn block(&self, by: By, value: &str) -> Result<models::Blocks> {
        block_api::block(self.configuration_ref(), by.as_str(), value)
            .await
            .map_err(|err| self.map_rest_error(err))
    }

    pub(crate) async fn blocks(
        &self,
        limit: i64,
        index: Option<i64>,
        sort: Option<SortDir>,
    ) -> Result<models::Blocks> {
        let sort = sort.map(|s| s.to_string());
        block_api::blocks(self.configuration_ref(), limit, index, sort.as_deref())
            .await
            .map_err(|err| self.map_rest_error(err))
    }

    pub(crate) async fn current_height(&self) -> Result<models::CurrentHeight> {
        block_api::current_height(self.configuration_ref())
            .await
            .map_err(|err| self.map_rest_error(err))
    }

    pub(crate) async fn fastbridge_info(&self) -> Result<models::RespGetFastBridgeInfo> {
        bridge_api::fastbridge_info(self.configuration_ref())
            .await
            .map_err(|err| self.map_rest_error(err))
    }

    pub(crate) async fn candlesticks(
        &self,
        market: MarketId,
        resolution: &str,
        start_timestamp: i64,
        end_timestamp: i64,
        count_back: i64,
        set_timestamp_to_end: Option<bool>,
    ) -> Result<models::Candlesticks> {
        candlestick_api::candlesticks(
            self.configuration_ref(),
            market.into(),
            resolution,
            start_timestamp,
            end_timestamp,
            count_back,
            set_timestamp_to_end,
        )
        .await
        .map_err(|err| self.map_rest_error(err))
    }

    pub(crate) async fn fundings(
        &self,
        market: MarketId,
        resolution: &str,
        start_timestamp: i64,
        end_timestamp: i64,
        count_back: i64,
    ) -> Result<models::Fundings> {
        candlestick_api::fundings(
            self.configuration_ref(),
            market.into(),
            resolution,
            start_timestamp,
            end_timestamp,
            count_back,
        )
        .await
        .map_err(|err| self.map_rest_error(err))
    }

    pub(crate) async fn funding_rates(&self) -> Result<models::FundingRates> {
        funding_api::funding_rates(self.configuration_ref())
            .await
            .map_err(|err| self.map_rest_error(err))
    }

    pub(crate) async fn transfer_fee_info(
        &self,
        account_index: i64,
        to_account_index: Option<AccountId>,
        auth: &str,
    ) -> Result<models::TransferFeeInfo> {
        info_api::transfer_fee_info(
            self.configuration_ref(),
            account_index,
            Some(auth),
            Some(auth),
            to_account_index.map(Into::into),
        )
        .await
        .map_err(|err| self.map_rest_error(err))
    }

    pub(crate) async fn withdrawal_delay(&self) -> Result<models::RespWithdrawalDelay> {
        info_api::withdrawal_delay(self.configuration_ref())
            .await
            .map_err(|err| self.map_rest_error(err))
    }

    pub(crate) async fn acknowledge_notification(
        &self,
        notification_id: &str,
        account_index: i64,
        auth: &str,
    ) -> Result<models::ResultCode> {
        notification_api::notification_ack(
            self.configuration_ref(),
            notification_id,
            account_index,
            Some(auth),
            Some(auth),
        )
        .await
        .map_err(|err| self.map_rest_error(err))
    }

    pub(crate) async fn account_active_orders(
        &self,
        account_index: i64,
        market: MarketId,
        auth: &str,
    ) -> Result<models::Orders> {
        order_api::account_active_orders(
            self.configuration_ref(),
            account_index,
            market.into(),
            Some(auth),
            Some(auth),
        )
        .await
        .map_err(|err| self.map_rest_error(err))
    }

    pub(crate) async fn account_inactive_orders(
        &self,
        account_index: i64,
        limit: i64,
        market: Option<MarketId>,
        ask_filter: Option<i32>,
        between_timestamps: Option<&str>,
        cursor: Option<&str>,
        auth: &str,
    ) -> Result<models::Orders> {
        order_api::account_inactive_orders(
            self.configuration_ref(),
            account_index,
            limit,
            Some(auth),
            Some(auth),
            market.map(Into::into),
            ask_filter,
            between_timestamps,
            cursor,
        )
        .await
        .map_err(|err| self.map_rest_error(err))
    }

    pub(crate) async fn exchange_stats(&self) -> Result<models::ExchangeStats> {
        order_api::exchange_stats(self.configuration_ref())
            .await
            .map_err(|err| self.map_rest_error(err))
    }

    pub(crate) async fn export(
        &self,
        export_type: &str,
        account_index: Option<i64>,
        auth: Option<&str>,
        market: Option<MarketId>,
    ) -> Result<models::ExportData> {
        order_api::export(
            self.configuration_ref(),
            export_type,
            auth,
            auth,
            account_index,
            market.map(Into::into),
        )
        .await
        .map_err(|err| self.map_rest_error(err))
    }

    pub(crate) async fn order_book_details(
        &self,
        market: Option<MarketId>,
    ) -> Result<models::OrderBookDetails> {
        order_api::order_book_details(self.configuration_ref(), market.map(Into::into))
            .await
            .map_err(|err| self.map_rest_error(err))
    }

    pub(crate) async fn order_books_metadata(
        &self,
        market: Option<MarketId>,
    ) -> Result<models::OrderBooks> {
        order_api::order_books(self.configuration_ref(), market.map(Into::into))
            .await
            .map_err(|err| self.map_rest_error(err))
    }

    pub(crate) async fn recent_trades(
        &self,
        market: MarketId,
        limit: i64,
    ) -> Result<models::Trades> {
        order_api::recent_trades(self.configuration_ref(), market.into(), limit)
            .await
            .map_err(|err| self.map_rest_error(err))
    }

    pub(crate) async fn trades(
        &self,
        sort_by: &TradeSort<'_>,
        limit: i64,
        market: Option<MarketId>,
        account_index: Option<i64>,
        order_index: Option<i64>,
        sort_dir: Option<SortDir>,
        cursor: Option<&str>,
        from: Option<i64>,
        ask_filter: Option<i32>,
        auth: Option<&str>,
    ) -> Result<models::Trades> {
        let sort_dir = sort_dir.map(|s| s.to_string());
        order_api::trades(
            self.configuration_ref(),
            sort_by.as_str(),
            limit,
            auth,
            auth,
            market.map(Into::into),
            account_index,
            order_index,
            sort_dir.as_deref(),
            cursor,
            from,
            ask_filter,
        )
        .await
        .map_err(|err| self.map_rest_error(err))
    }

    pub(crate) async fn account_transactions(
        &self,
        limit: i64,
        account_value: &str,
        index: Option<i64>,
        types: Option<Vec<i32>>,
        auth: &str,
    ) -> Result<models::Txs> {
        transaction_api::account_txs(
            self.configuration_ref(),
            limit,
            "index",
            account_value,
            Some(auth),
            index,
            types,
            Some(auth),
        )
        .await
        .map_err(|err| self.map_rest_error(err))
    }

    pub(crate) async fn block_transactions(&self, by: By, value: &str) -> Result<models::Txs> {
        transaction_api::block_txs(self.configuration_ref(), by.as_str(), value)
            .await
            .map_err(|err| self.map_rest_error(err))
    }

    pub(crate) async fn deposit_history(
        &self,
        account_index: i64,
        l1_address: &str,
        cursor: Option<&str>,
        filter: Option<&str>,
        auth: &str,
    ) -> Result<models::DepositHistory> {
        transaction_api::deposit_history(
            self.configuration_ref(),
            account_index,
            l1_address,
            Some(auth),
            Some(auth),
            cursor,
            filter,
        )
        .await
        .map_err(|err| self.map_rest_error(err))
    }

    pub(crate) async fn transfer_history(
        &self,
        account_index: i64,
        cursor: Option<&str>,
        auth: &str,
    ) -> Result<models::TransferHistory> {
        transaction_api::transfer_history(
            self.configuration_ref(),
            account_index,
            Some(auth),
            Some(auth),
            cursor,
        )
        .await
        .map_err(|err| self.map_rest_error(err))
    }

    pub(crate) async fn withdraw_history(
        &self,
        account_index: i64,
        cursor: Option<&str>,
        filter: Option<&str>,
        auth: &str,
    ) -> Result<models::WithdrawHistory> {
        transaction_api::withdraw_history(
            self.configuration_ref(),
            account_index,
            Some(auth),
            Some(auth),
            cursor,
            filter,
        )
        .await
        .map_err(|err| self.map_rest_error(err))
    }

    pub(crate) async fn next_nonce(
        &self,
        account_index: i64,
        api_key_index: ApiKeyIndex,
    ) -> Result<models::NextNonce> {
        transaction_api::next_nonce(
            self.configuration_ref(),
            account_index,
            api_key_index.into(),
        )
        .await
        .map_err(|err| self.map_rest_error(err))
    }

    pub(crate) async fn transaction(&self, by: By, value: &str) -> Result<models::EnrichedTx> {
        transaction_api::tx(self.configuration_ref(), by.as_str(), value)
            .await
            .map_err(|err| self.map_rest_error(err))
    }

    pub(crate) async fn transaction_from_l1_hash(&self, hash: &str) -> Result<models::EnrichedTx> {
        transaction_api::tx_from_l1_tx_hash(self.configuration_ref(), hash)
            .await
            .map_err(|err| self.map_rest_error(err))
    }

    pub(crate) async fn transactions(&self, limit: i64, index: Option<i64>) -> Result<models::Txs> {
        transaction_api::txs(self.configuration_ref(), limit, index)
            .await
            .map_err(|err| self.map_rest_error(err))
    }

    pub(crate) fn map_rest_error<E>(&self, err: apis::Error<E>) -> Error
    where
        E: serde::Serialize + fmt::Debug,
    {
        let error = match err {
            apis::Error::ResponseError(resp) => {
                let status = resp.status.as_u16();
                if let Ok(server) = serde_json::from_str::<ServerErr>(&resp.content) {
                    if status == 429 {
                        return Error::RateLimited {
                            retry_after: server.retry_after,
                        };
                    }
                    Error::Server {
                        status,
                        message: server.message,
                        code: server.code,
                    }
                } else {
                    Error::Http {
                        status,
                        body: resp.content,
                    }
                }
            }
            apis::Error::Reqwest(err) => {
                let status = err.status().map(|s| s.as_u16()).unwrap_or_default();
                Error::Http {
                    status,
                    body: err.to_string(),
                }
            }
            apis::Error::Serde(err) => Error::Http {
                status: 500,
                body: err.to_string(),
            },
            apis::Error::Io(err) => Error::Http {
                status: 500,
                body: err.to_string(),
            },
        };

        if Self::is_unauthorized(&error) {
            let cache = self.auth.clone();
            if tokio::runtime::Handle::try_current().is_ok() {
                tokio::spawn(async move {
                    cache.invalidate().await;
                });
            } else {
                cache.invalidate_blocking();
            }
        }

        error
    }

    pub(crate) fn is_unauthorized(error: &Error) -> bool {
        matches!(error, Error::Http { status, .. } if *status == 401)
            || matches!(error, Error::Server { status, .. } if *status == 401)
    }
}
