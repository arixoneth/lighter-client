use std::{
    collections::HashMap,
    fmt,
    marker::PhantomData,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use futures_util::Stream;

use super::{
    auth::AuthCache,
    errors::{Error, Result},
    pagination::paginate_items,
    params::{
        By, CandleResolution, FundingSide, HistoryFilter, OrderFilter, PageCursor, PnlResolution,
        PoolFilter, SortDir, TimeRange, Timestamp, TradeSort,
    },
    queries::{HistoryQuery, InactiveOrdersQuery, TradesQuery},
    rest::RestClient,
};
use crate::{
    apis::configuration,
    models,
    nonce_manager::NonceManagerType,
    signer_client::{SignedPayload, SignerClient},
    transactions,
    types::{AccountId, ApiKeyIndex, BaseQty, Expiry, MarketId, Nonce, Price, UsdcAmount},
    ws_client::{WsBuilder, WsConfig},
};

/// Configuration options for [`LighterClient`].
#[derive(Debug, Clone)]
pub struct LighterClientOptions {
    max_api_key_index: Option<ApiKeyIndex>,
    extra_private_keys: HashMap<ApiKeyIndex, String>,
    nonce_management: NonceManagerType,
    signer_library_path: Option<PathBuf>,
    websocket: Option<WsConfig>,
}

impl Default for LighterClientOptions {
    fn default() -> Self {
        Self {
            max_api_key_index: None,
            extra_private_keys: HashMap::new(),
            nonce_management: NonceManagerType::Optimistic,
            signer_library_path: None,
            websocket: None,
        }
    }
}

impl LighterClientOptions {
    pub fn with_max_api_key_index(mut self, index: ApiKeyIndex) -> Self {
        self.max_api_key_index = Some(index);
        self
    }

    pub fn with_extra_private_key(mut self, index: ApiKeyIndex, value: impl Into<String>) -> Self {
        self.extra_private_keys.insert(index, value.into());
        self
    }

    pub fn with_nonce_management(mut self, nonce_management: NonceManagerType) -> Self {
        self.nonce_management = nonce_management;
        self
    }

    pub fn with_signer_library_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.signer_library_path = Some(path.into());
        self
    }

    pub fn with_websocket(mut self, config: WsConfig) -> Self {
        self.websocket = Some(config);
        self
    }
}

/// Top level wrapper that exposes a high level API for both REST and
/// transaction signing capabilities.
pub struct LighterClient {
    rest: RestClient,
    signer: Option<SignerClient>,
    ws_cfg: WsConfig,
    auth: AuthCache,
    account_id: Option<AccountId>,
    opts: LighterClientOptions,
}

impl LighterClient {
    /// Create a new builder used to configure a [`LighterClient`].
    pub fn builder() -> LighterClientBuilder {
        LighterClientBuilder {
            api_url: None,
            private_key: None,
            api_key_index: None,
            account_index: None,
            options: LighterClientOptions::default(),
        }
    }

    /// Construct a new [`LighterClient`] instance without configuring an account.
    pub async fn new(api_url: impl Into<String>) -> Result<Self> {
        Self::new_with_options(api_url, LighterClientOptions::default()).await
    }

    /// Construct a new [`LighterClient`] instance allowing custom options.
    pub async fn new_with_options(
        api_url: impl Into<String>,
        options: LighterClientOptions,
    ) -> Result<Self> {
        let auth = AuthCache::default();
        let rest = RestClient::new(api_url, auth.clone());
        let ws_cfg = options.websocket.clone().unwrap_or_else(WsConfig::default);

        Ok(Self {
            rest,
            signer: None,
            ws_cfg,
            auth,
            account_id: None,
            opts: options,
        })
    }

    /// Underlying REST configuration. This can be used to call any of the
    /// generated REST APIs directly when a specialised helper is not present.
    pub fn configuration(&self) -> configuration::Configuration {
        self.rest.configuration()
    }

    /// Primary account identifier managed by this client.
    pub fn account_id(&self) -> Option<AccountId> {
        self.account_id
    }

    /// Access the inner [`SignerClient`] for advanced use cases.
    pub fn signer(&self) -> Option<&SignerClient> {
        self.signer.as_ref()
    }

    /// Access account scoped REST helpers.
    pub fn account(&self) -> AccountHandle<'_> {
        AccountHandle { c: self }
    }

    /// Access block REST helpers.
    pub fn blocks(&self) -> BlocksHandle<'_> {
        BlocksHandle { c: self }
    }

    /// Access bridge REST helpers.
    pub fn bridge(&self) -> BridgeHandle<'_> {
        BridgeHandle { c: self }
    }

    /// Access candlestick REST helpers.
    pub fn candles(&self) -> CandlesHandle<'_> {
        CandlesHandle { c: self }
    }

    /// Access funding REST helpers.
    pub fn funding(&self) -> FundingHandle<'_> {
        FundingHandle { c: self }
    }

    /// Access general information REST helpers.
    pub fn info(&self) -> InfoHandle<'_> {
        InfoHandle { c: self }
    }

    /// Access notification REST helpers.
    pub fn notifications(&self) -> NotificationsHandle<'_> {
        NotificationsHandle { c: self }
    }

    /// Access public order and trade REST helpers.
    pub fn orders(&self) -> OrdersHandle<'_> {
        OrdersHandle { c: self }
    }

    /// Access transaction REST helpers.
    pub fn transactions(&self) -> TransactionsHandle<'_> {
        TransactionsHandle { c: self }
    }

    /// Create a fluent order builder scoped to the provided market.
    pub fn order(&self, market: MarketId) -> OrderBuilder<'_, OrderStateInit> {
        OrderBuilder::new(self, market)
    }

    pub fn order_batch(&self) -> OrderBatchBuilder<'_> {
        OrderBatchBuilder::new()
    }

    /// Create a cancellation builder for a specific order.
    pub fn cancel(&self, market: MarketId, order_index: i64) -> CancelOrderBuilder<'_> {
        CancelOrderBuilder::new(self, market, order_index)
    }

    /// Create a builder that cancels all resting orders for the configured account.
    pub fn cancel_all(&self) -> CancelAllBuilder<'_> {
        CancelAllBuilder::new(self)
    }

    /// Create a withdrawal builder for the active account.
    pub fn withdraw(&self, amount: UsdcAmount) -> WithdrawBuilder<'_> {
        WithdrawBuilder::new(self, amount)
    }

    /// Create a websocket builder scoped to this client.
    pub fn ws(&self) -> WsBuilder<'_> {
        WsBuilder::new(self)
    }

    /// Create an auth token (with an optional custom expiry).
    pub fn create_auth_token(&self, expiry: Option<Expiry>) -> Result<String> {
        let expiry = expiry.map(|value| value.into_unix());
        let token = self.signer_ref()?.create_auth_token_with_expiry(expiry)?;
        Ok(token.token)
    }

    /// Configure the client with an authenticated account in place.
    pub async fn configure_account(
        &mut self,
        private_key: impl AsRef<str>,
        api_key_index: ApiKeyIndex,
        account_index: AccountId,
    ) -> Result<()> {
        let extra_keys = if self.opts.extra_private_keys.is_empty() {
            None
        } else {
            let mut keys = HashMap::with_capacity(self.opts.extra_private_keys.len());
            for (index, key) in &self.opts.extra_private_keys {
                keys.insert((*index).into(), key.clone());
            }
            Some(keys)
        };

        let signer = SignerClient::new(
            self.rest.base_path().to_string(),
            private_key,
            api_key_index.into(),
            account_index.into(),
            self.opts.max_api_key_index.map(Into::into),
            extra_keys,
            self.opts.nonce_management,
            self.opts.signer_library_path.as_deref(),
        )
        .await?;

        self.rest.set_configuration(signer.configuration());
        self.auth.invalidate().await;
        self.signer = Some(signer);
        self.account_id = Some(account_index);

        Ok(())
    }

    /// Configure the client with an authenticated account.
    pub async fn with_account(
        mut self,
        private_key: impl AsRef<str>,
        api_key_index: ApiKeyIndex,
        account_index: AccountId,
    ) -> Result<Self> {
        self.configure_account(private_key, api_key_index, account_index)
            .await?;
        Ok(self)
    }

    fn signer_ref(&self) -> Result<&SignerClient> {
        self.signer.as_ref().ok_or(Error::NotAuthenticated)
    }

    fn require_account(&self) -> Result<AccountId> {
        self.account_id.ok_or(Error::NotAuthenticated)
    }

    fn require_account_i64(&self) -> Result<i64> {
        self.require_account().map(Into::into)
    }

    fn require_account_value(&self) -> Result<String> {
        self.require_account_i64().map(|value| value.to_string())
    }

    async fn auth_header(&self) -> Result<String> {
        let signer = self.signer_ref()?;
        self.auth.header(signer).await
    }

    pub(crate) fn websocket_config(&self) -> &WsConfig {
        &self.ws_cfg
    }

    pub(crate) fn rest_base_path(&self) -> &str {
        self.rest.base_path()
    }
}

/// Account scoped helper providing access to authenticated endpoints.
pub struct AccountHandle<'a> {
    c: &'a LighterClient,
}

impl<'a> AccountHandle<'a> {
    pub async fn details(&self) -> Result<models::DetailedAccounts> {
        let account_value = self.c.require_account_value()?;
        self.c.rest.account_details(&account_value).await
    }

    pub async fn limits(&self) -> Result<models::AccountLimits> {
        let account_index = self.c.require_account_i64()?;
        let auth = self.c.auth_header().await?;
        self.c
            .rest
            .account_limits(account_index, auth.as_str())
            .await
    }

    pub async fn metadata(&self) -> Result<models::AccountMetadatas> {
        let account_value = self.c.require_account_value()?;
        let auth = self.c.auth_header().await?;
        self.c
            .rest
            .account_metadata(&account_value, auth.as_str())
            .await
    }

    pub async fn api_keys(
        &self,
        api_key_index: Option<ApiKeyIndex>,
    ) -> Result<models::AccountApiKeys> {
        let account_index = self.c.require_account_i64()?;
        self.c.rest.api_keys(account_index, api_key_index).await
    }

    pub async fn change_tier(&self, new_tier: &str) -> Result<models::RespChangeAccountTier> {
        let account_index = self.c.require_account_i64()?;
        let auth = self.c.auth_header().await?;
        self.c
            .rest
            .change_account_tier(account_index, new_tier, auth.as_str())
            .await
    }

    pub async fn l1_metadata(&self, l1_address: &str) -> Result<models::L1Metadata> {
        let auth = self.c.auth_header().await?;
        self.c.rest.l1_metadata(l1_address, auth.as_str()).await
    }

    pub async fn liquidations(
        &self,
        limit: i64,
        market: Option<MarketId>,
        cursor: Option<PageCursor<'_>>,
    ) -> Result<models::LiquidationInfos> {
        let account_index = self.c.require_account_i64()?;
        let auth = self.c.auth_header().await?;
        self.c
            .rest
            .liquidations(
                account_index,
                limit,
                market,
                cursor.as_ref().map(PageCursor::as_str),
                auth.as_str(),
            )
            .await
    }

    pub async fn pnl(
        &self,
        resolution: PnlResolution<'_>,
        range: TimeRange,
        count_back: i64,
        ignore_transfers: Option<bool>,
    ) -> Result<models::AccountPnL> {
        let account_value = self.c.require_account_value()?;
        let auth = self.c.auth_header().await?;
        let (start, end) = range.bounds();
        self.c
            .rest
            .account_pnl(
                &account_value,
                resolution.as_str(),
                start,
                end,
                count_back,
                ignore_transfers,
                auth.as_str(),
            )
            .await
    }

    pub async fn position_funding(
        &self,
        limit: i64,
        market: Option<MarketId>,
        cursor: Option<PageCursor<'_>>,
        side: Option<FundingSide>,
    ) -> Result<models::PositionFundings> {
        let account_index = self.c.require_account_i64()?;
        let auth = self.c.auth_header().await?;
        self.c
            .rest
            .position_funding(
                account_index,
                limit,
                market,
                cursor.as_ref().map(PageCursor::as_str),
                side.map(FundingSide::as_str),
                auth.as_str(),
            )
            .await
    }

    pub async fn public_pools(
        &self,
        index: i64,
        limit: i64,
        filter: Option<PoolFilter<'_>>,
    ) -> Result<models::PublicPools> {
        let account_index = self.c.require_account_i64()?;
        let auth = self.c.auth_header().await?;
        let filter = filter.as_ref().map(PoolFilter::as_str);
        self.c
            .rest
            .public_pools(index, limit, filter, account_index, auth.as_str())
            .await
    }

    pub async fn public_pools_metadata(
        &self,
        index: i64,
        limit: i64,
        filter: Option<PoolFilter<'_>>,
    ) -> Result<models::RespPublicPoolsMetadata> {
        let account_index = self.c.require_account_i64()?;
        let auth = self.c.auth_header().await?;
        let filter = filter.as_ref().map(PoolFilter::as_str);
        self.c
            .rest
            .public_pools_metadata(index, limit, filter, account_index, auth.as_str())
            .await
    }

    pub async fn by_l1(&self, l1_address: &str) -> Result<models::SubAccounts> {
        self.c.rest.accounts_by_l1_address(l1_address).await
    }

    pub async fn active_orders(&self, market: MarketId) -> Result<models::Orders> {
        let account_index = self.c.require_account_i64()?;
        let auth = self.c.auth_header().await?;
        self.c
            .rest
            .account_active_orders(account_index, market, auth.as_str())
            .await
    }

    pub async fn inactive_orders(&self, query: InactiveOrdersQuery<'_>) -> Result<models::Orders> {
        let cursor = query.cursor_ref().map(PageCursor::as_str);
        self.request_inactive_orders_with_cursor(&query, cursor)
            .await
    }

    async fn request_inactive_orders_with_cursor(
        &self,
        query: &InactiveOrdersQuery<'_>,
        cursor: Option<&str>,
    ) -> Result<models::Orders> {
        let account_index = self.c.require_account_i64()?;
        let auth = self.c.auth_header().await?;
        let between = query.between_range().map(TimeRange::as_query);
        self.c
            .rest
            .account_inactive_orders(
                account_index,
                query.limit(),
                query.market_id(),
                query.order_filter().as_query(),
                between.as_deref(),
                cursor,
                auth.as_str(),
            )
            .await
    }

    pub fn inactive_orders_stream<'b>(
        &'b self,
        query: InactiveOrdersQuery<'b>,
    ) -> Result<impl Stream<Item = Result<models::Order>> + 'b> {
        self.c.require_account_i64()?;
        let initial_cursor = query.cursor_ref().map(|cursor| cursor.clone().into_owned());
        let handle = self;
        Ok(paginate_items(initial_cursor, move |cursor| {
            let query = query.clone();
            async move {
                handle
                    .request_inactive_orders_with_cursor(&query, cursor.as_deref())
                    .await
            }
        }))
    }

    pub async fn trades(&self, query: TradesQuery<'_>) -> Result<models::Trades> {
        let cursor = query.cursor_ref().map(PageCursor::as_str);
        self.request_trades_with_cursor(&query, cursor).await
    }

    async fn request_trades_with_cursor(
        &self,
        query: &TradesQuery<'_>,
        cursor: Option<&str>,
    ) -> Result<models::Trades> {
        let account_index = Some(self.c.require_account_i64()?);
        let auth = self.c.auth_header().await?;
        self.c
            .rest
            .trades(
                query.sort(),
                query.limit(),
                query.market_id(),
                account_index,
                query.order_index_value(),
                query.sort_direction(),
                cursor,
                query.from_timestamp().map(Timestamp::into_unix),
                query.order_filter().as_query(),
                Some(auth.as_str()),
            )
            .await
    }

    pub fn trades_stream<'b>(
        &'b self,
        query: TradesQuery<'b>,
    ) -> Result<impl Stream<Item = Result<models::Trade>> + 'b> {
        self.c.require_account_i64()?;
        let initial_cursor = query.cursor_ref().map(|cursor| cursor.clone().into_owned());
        let handle = self;
        Ok(paginate_items(initial_cursor, move |cursor| {
            let query = query.clone();
            async move {
                handle
                    .request_trades_with_cursor(&query, cursor.as_deref())
                    .await
            }
        }))
    }

    pub async fn transactions(
        &self,
        limit: i64,
        index: Option<i64>,
        types: Option<Vec<i32>>,
    ) -> Result<models::Txs> {
        let account_value = self.c.require_account_value()?;
        let auth = self.c.auth_header().await?;
        self.c
            .rest
            .account_transactions(limit, &account_value, index, types, auth.as_str())
            .await
    }

    pub async fn deposit_history(
        &self,
        l1_address: &str,
        query: HistoryQuery<'_>,
    ) -> Result<models::DepositHistory> {
        let filter = query.filter_ref().map(HistoryFilter::as_str);
        let cursor = query.cursor_ref().map(PageCursor::as_str);
        self.request_deposit_history_with_cursor(l1_address, filter, cursor)
            .await
    }

    pub async fn transfer_history(
        &self,
        query: HistoryQuery<'_>,
    ) -> Result<models::TransferHistory> {
        let cursor = query.cursor_ref().map(PageCursor::as_str);
        self.request_transfer_history_with_cursor(cursor).await
    }

    pub async fn withdraw_history(
        &self,
        query: HistoryQuery<'_>,
    ) -> Result<models::WithdrawHistory> {
        let filter = query.filter_ref().map(HistoryFilter::as_str);
        let cursor = query.cursor_ref().map(PageCursor::as_str);
        self.request_withdraw_history_with_cursor(filter, cursor)
            .await
    }

    async fn request_deposit_history_with_cursor(
        &self,
        l1_address: &str,
        filter: Option<&str>,
        cursor: Option<&str>,
    ) -> Result<models::DepositHistory> {
        let account_index = self.c.require_account_i64()?;
        let auth = self.c.auth_header().await?;
        self.c
            .rest
            .deposit_history(account_index, l1_address, cursor, filter, auth.as_str())
            .await
    }

    async fn request_transfer_history_with_cursor(
        &self,
        cursor: Option<&str>,
    ) -> Result<models::TransferHistory> {
        let account_index = self.c.require_account_i64()?;
        let auth = self.c.auth_header().await?;
        self.c
            .rest
            .transfer_history(account_index, cursor, auth.as_str())
            .await
    }

    async fn request_withdraw_history_with_cursor(
        &self,
        filter: Option<&str>,
        cursor: Option<&str>,
    ) -> Result<models::WithdrawHistory> {
        let account_index = self.c.require_account_i64()?;
        let auth = self.c.auth_header().await?;
        self.c
            .rest
            .withdraw_history(account_index, cursor, filter, auth.as_str())
            .await
    }

    pub fn deposit_history_stream<'b>(
        &'b self,
        l1_address: &'b str,
        query: HistoryQuery<'b>,
    ) -> Result<impl Stream<Item = Result<models::DepositHistoryItem>> + 'b> {
        self.c.require_account_i64()?;
        let initial_cursor = query.cursor_ref().map(|cursor| cursor.clone().into_owned());
        let filter = query.filter_ref().map(|filter| filter.as_str().to_string());
        let handle = self;
        Ok(paginate_items(initial_cursor, move |cursor| {
            let filter = filter.clone();
            async move {
                handle
                    .request_deposit_history_with_cursor(
                        l1_address,
                        filter.as_deref(),
                        cursor.as_deref(),
                    )
                    .await
            }
        }))
    }

    pub fn transfer_history_stream<'b>(
        &'b self,
        query: HistoryQuery<'b>,
    ) -> Result<impl Stream<Item = Result<models::TransferHistoryItem>> + 'b> {
        self.c.require_account_i64()?;
        let initial_cursor = query.cursor_ref().map(|cursor| cursor.clone().into_owned());
        let handle = self;
        Ok(paginate_items(initial_cursor, move |cursor| async move {
            handle
                .request_transfer_history_with_cursor(cursor.as_deref())
                .await
        }))
    }

    pub fn withdraw_history_stream<'b>(
        &'b self,
        query: HistoryQuery<'b>,
    ) -> Result<impl Stream<Item = Result<models::WithdrawHistoryItem>> + 'b> {
        self.c.require_account_i64()?;
        let initial_cursor = query.cursor_ref().map(|cursor| cursor.clone().into_owned());
        let filter = query.filter_ref().map(|filter| filter.as_str().to_string());
        let handle = self;
        Ok(paginate_items(initial_cursor, move |cursor| {
            let filter = filter.clone();
            async move {
                handle
                    .request_withdraw_history_with_cursor(filter.as_deref(), cursor.as_deref())
                    .await
            }
        }))
    }

    pub async fn next_nonce(&self, api_key_index: ApiKeyIndex) -> Result<models::NextNonce> {
        let account_index = self.c.require_account_i64()?;
        self.c.rest.next_nonce(account_index, api_key_index).await
    }

    pub async fn transfer_fee_info(
        &self,
        to_account_index: Option<AccountId>,
    ) -> Result<models::TransferFeeInfo> {
        let account_index = self.c.require_account_i64()?;
        let auth = self.c.auth_header().await?;
        self.c
            .rest
            .transfer_fee_info(account_index, to_account_index, auth.as_str())
            .await
    }
}

/// Block helper exposing block REST endpoints.
pub struct BlocksHandle<'a> {
    c: &'a LighterClient,
}

impl<'a> BlocksHandle<'a> {
    pub async fn one(&self, by: By, value: impl AsRef<str>) -> Result<models::Blocks> {
        self.c.rest.block(by, value.as_ref()).await
    }

    pub async fn list(
        &self,
        limit: i64,
        index: Option<i64>,
        sort: Option<SortDir>,
    ) -> Result<models::Blocks> {
        self.c.rest.blocks(limit, index, sort).await
    }

    pub async fn current_height(&self) -> Result<models::CurrentHeight> {
        self.c.rest.current_height().await
    }

    pub async fn transactions(&self, by: By, value: impl AsRef<str>) -> Result<models::Txs> {
        self.c.rest.block_transactions(by, value.as_ref()).await
    }
}

/// Bridge helper exposing cross-chain endpoints.
pub struct BridgeHandle<'a> {
    c: &'a LighterClient,
}

impl<'a> BridgeHandle<'a> {
    pub async fn fastbridge_info(&self) -> Result<models::RespGetFastBridgeInfo> {
        self.c.rest.fastbridge_info().await
    }
}

/// Candlestick helper exposing price and funding candles.
pub struct CandlesHandle<'a> {
    c: &'a LighterClient,
}

impl<'a> CandlesHandle<'a> {
    pub async fn price(
        &self,
        market: MarketId,
        resolution: CandleResolution<'_>,
        range: TimeRange,
        count_back: i64,
        set_timestamp_to_end: Option<bool>,
    ) -> Result<models::Candlesticks> {
        let (start, end) = range.bounds();
        self.c
            .rest
            .candlesticks(
                market,
                resolution.as_str(),
                start,
                end,
                count_back,
                set_timestamp_to_end,
            )
            .await
    }

    pub async fn funding(
        &self,
        market: MarketId,
        resolution: CandleResolution<'_>,
        range: TimeRange,
        count_back: i64,
    ) -> Result<models::Fundings> {
        let (start, end) = range.bounds();
        self.c
            .rest
            .fundings(market, resolution.as_str(), start, end, count_back)
            .await
    }
}

/// Funding helper for rates endpoints.
pub struct FundingHandle<'a> {
    c: &'a LighterClient,
}

impl<'a> FundingHandle<'a> {
    pub async fn rates(&self) -> Result<models::FundingRates> {
        self.c.rest.funding_rates().await
    }
}

/// Info helper for general informational endpoints.
pub struct InfoHandle<'a> {
    c: &'a LighterClient,
}

impl<'a> InfoHandle<'a> {
    pub async fn withdrawal_delay(&self) -> Result<models::RespWithdrawalDelay> {
        self.c.rest.withdrawal_delay().await
    }
}

/// Notification helper for account scoped notifications.
pub struct NotificationsHandle<'a> {
    c: &'a LighterClient,
}

impl<'a> NotificationsHandle<'a> {
    pub async fn acknowledge(&self, notification_id: &str) -> Result<models::ResultCode> {
        let account_index = self.c.require_account_i64()?;
        let auth = self.c.auth_header().await?;
        self.c
            .rest
            .acknowledge_notification(notification_id, account_index, auth.as_str())
            .await
    }
}

/// Orders helper for public order and trade endpoints.
pub struct OrdersHandle<'a> {
    c: &'a LighterClient,
}

impl<'a> OrdersHandle<'a> {
    pub async fn book(&self, market: MarketId, limit: i64) -> Result<models::OrderBookOrders> {
        self.c.rest.order_book(market, limit).await
    }

    pub async fn book_details(&self, market: Option<MarketId>) -> Result<models::OrderBookDetails> {
        self.c.rest.order_book_details(market).await
    }

    pub async fn books_metadata(&self, market: Option<MarketId>) -> Result<models::OrderBooks> {
        self.c.rest.order_books_metadata(market).await
    }

    pub async fn recent(&self, market: MarketId, limit: i64) -> Result<models::Trades> {
        self.c.rest.recent_trades(market, limit).await
    }

    pub async fn trades(
        &self,
        sort_by: TradeSort<'_>,
        limit: i64,
        market: Option<MarketId>,
        sort_dir: Option<SortDir>,
        cursor: Option<PageCursor<'_>>,
        from: Option<i64>,
        ask_filter: Option<i32>,
    ) -> Result<models::Trades> {
        let account_index = Some(self.c.require_account_i64()?);
        let auth = self.c.auth_header().await?;
        self.c
            .rest
            .trades(
                &sort_by,
                limit,
                market,
                account_index,
                None,
                sort_dir,
                cursor.as_ref().map(PageCursor::as_str),
                from,
                ask_filter,
                Some(auth.as_str()),
            )
            .await
    }

    pub async fn exchange_stats(&self) -> Result<models::ExchangeStats> {
        self.c.rest.exchange_stats().await
    }

    pub async fn export(
        &self,
        export_type: &str,
        market: Option<MarketId>,
        include_account: bool,
    ) -> Result<models::ExportData> {
        let (account_index, auth) = if include_account {
            let auth = self.c.auth_header().await?;
            let account_index = Some(self.c.require_account_i64()?);
            (account_index, Some(auth))
        } else {
            (None, None)
        };

        self.c
            .rest
            .export(export_type, account_index, auth.as_deref(), market)
            .await
    }
}

/// Transaction helper for public transaction endpoints.
pub struct TransactionsHandle<'a> {
    c: &'a LighterClient,
}

impl<'a> TransactionsHandle<'a> {
    pub async fn one(&self, by: By, value: impl AsRef<str>) -> Result<models::EnrichedTx> {
        self.c.rest.transaction(by, value.as_ref()).await
    }

    pub async fn by_l1_hash(&self, hash: impl AsRef<str>) -> Result<models::EnrichedTx> {
        self.c.rest.transaction_from_l1_hash(hash.as_ref()).await
    }

    pub async fn list(&self, limit: i64, index: Option<i64>) -> Result<models::Txs> {
        self.c.rest.transactions(limit, index).await
    }
}

/// Builder returned by [`LighterClient::builder`] that makes it easier to
/// configure the client.
pub struct LighterClientBuilder {
    api_url: Option<String>,
    private_key: Option<String>,
    api_key_index: Option<ApiKeyIndex>,
    account_index: Option<AccountId>,
    options: LighterClientOptions,
}

impl LighterClientBuilder {
    pub fn api_url(mut self, url: impl Into<String>) -> Self {
        self.api_url = Some(url.into());
        self
    }

    pub fn private_key(mut self, private_key: impl Into<String>) -> Self {
        self.private_key = Some(private_key.into());
        self
    }

    pub fn api_key_index(mut self, index: impl Into<ApiKeyIndex>) -> Self {
        self.api_key_index = Some(index.into());
        self
    }

    pub fn account_index(mut self, index: impl Into<AccountId>) -> Self {
        self.account_index = Some(index.into());
        self
    }

    pub fn nonce_management(mut self, nonce_management: NonceManagerType) -> Self {
        self.options.nonce_management = nonce_management;
        self
    }

    pub fn signer_library_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.options.signer_library_path = Some(path.into());
        self
    }

    pub fn websocket(mut self, host: impl Into<String>, path: impl Into<String>) -> Self {
        let mut config = self
            .options
            .websocket
            .clone()
            .unwrap_or_else(WsConfig::default);
        config.host = host.into();
        config.path = path.into();
        self.options.websocket = Some(config);
        self
    }

    pub fn extra_private_key(
        mut self,
        index: impl Into<ApiKeyIndex>,
        private_key: impl Into<String>,
    ) -> Self {
        self.options
            .extra_private_keys
            .insert(index.into(), private_key.into());
        self
    }

    pub fn max_api_key_index(mut self, index: impl Into<ApiKeyIndex>) -> Self {
        self.options.max_api_key_index = Some(index.into());
        self
    }

    pub async fn build(self) -> Result<LighterClient> {
        let api_url = self.api_url.ok_or(Error::InvalidConfig {
            field: "api_url",
            why: "must be provided",
        })?;

        let mut client = LighterClient::new_with_options(api_url, self.options).await?;

        if let (Some(private_key), Some(api_key_index), Some(account_index)) =
            (self.private_key, self.api_key_index, self.account_index)
        {
            client
                .configure_account(private_key, api_key_index, account_index)
                .await?;
        }

        Ok(client)
    }
}

/// Generic submission wrapper containing the signed transaction payload and the
/// API response.
#[derive(Debug, Clone)]
pub struct Submission<T> {
    payload: T,
    response: models::RespSendTx,
}

impl<T> Submission<T> {
    fn new(payload: T, response: models::RespSendTx) -> Self {
        Self { payload, response }
    }

    pub fn payload(&self) -> &T {
        &self.payload
    }

    pub fn response(&self) -> &models::RespSendTx {
        &self.response
    }

    pub fn into_payload(self) -> T {
        self.payload
    }

    pub fn into_parts(self) -> (T, models::RespSendTx) {
        (self.payload, self.response)
    }
}

/// Order side representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderSide {
    Bid,
    Ask,
}

impl OrderSide {
    fn is_ask(self) -> bool {
        matches!(self, OrderSide::Ask)
    }

    fn opposite(self) -> OrderSide {
        match self {
            OrderSide::Bid => OrderSide::Ask,
            OrderSide::Ask => OrderSide::Bid,
        }
    }
}

impl fmt::Display for OrderSide {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrderSide::Bid => write!(f, "Bid"),
            OrderSide::Ask => write!(f, "Ask"),
        }
    }
}

/// Supported order time in force strategies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderTimeInForce {
    ImmediateOrCancel,
    GoodTillTime,
    PostOnly,
}

#[derive(Clone, Copy, Debug)]
enum ClientOrderIdStrategy {
    Manual(i64),
    Auto,
}

#[derive(Clone, Copy, Debug)]
enum SlippageStrategy {
    Enforced {
        max_slippage: f64,
        ideal_price: Option<Price>,
    },
    CheckOnly {
        max_slippage: f64,
        ideal_price: Option<Price>,
    },
}

fn generate_client_order_id() -> i64 {
    let micros = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros();
    (micros % i64::MAX as u128) as i64
}

struct OrderBuilderState<'a> {
    client: &'a LighterClient,
    market: MarketId,
    client_order_id: ClientOrderIdStrategy,
    side: Option<OrderSide>,
    qty: Option<BaseQty>,
    tif: Option<OrderTimeInForce>,
    reduce_only: bool,
    expiry: Option<Expiry>,
    nonce: Option<Nonce>,
    api_key_override: Option<ApiKeyIndex>,
    kind: Option<OrderKind>,
    avg_execution_override: Option<Price>,
    slippage: Option<SlippageStrategy>,
    price_offset_ticks: i64,
}

impl<'a> OrderBuilderState<'a> {
    fn new(client: &'a LighterClient, market: MarketId) -> Self {
        Self {
            client,
            market,
            client_order_id: ClientOrderIdStrategy::Manual(0),
            side: None,
            qty: None,
            tif: None,
            reduce_only: false,
            expiry: None,
            nonce: None,
            api_key_override: None,
            kind: None,
            avg_execution_override: None,
            slippage: None,
            price_offset_ticks: 0,
        }
    }

    fn with_side(&mut self, side: OrderSide) {
        self.side = Some(side);
    }

    fn with_qty(&mut self, qty: BaseQty) {
        self.qty = Some(qty);
    }

    fn with_kind(&mut self, kind: OrderKind) {
        self.kind = Some(kind);
    }

    fn validate(&self) -> Result<()> {
        if self.side.is_none() {
            return Err(Error::InvalidConfig {
                field: "side",
                why: "must be specified via buy() or sell()",
            });
        }

        if self.qty.is_none() {
            return Err(Error::InvalidConfig {
                field: "qty",
                why: "must be provided via qty()",
            });
        }

        let kind = self.kind.as_ref().ok_or(Error::InvalidConfig {
            field: "kind",
            why: "must be selected via limit(), market(), or conditional helpers",
        })?;

        if let Some(OrderTimeInForce::PostOnly) = self.tif {
            if matches!(
                kind,
                OrderKind::Market { .. }
                    | OrderKind::StopLossMarket { .. }
                    | OrderKind::TakeProfitMarket { .. }
            ) {
                return Err(Error::InvalidConfig {
                    field: "time_in_force",
                    why: "post-only is unsupported for this order type",
                });
            }
        }

        if self.slippage.is_some()
            && !matches!(kind, OrderKind::Market { .. })
            && !matches!(self.kind, Some(OrderKind::Market { .. }))
        {
            return Err(Error::InvalidConfig {
                field: "slippage",
                why: "slippage controls only apply to market orders",
            });
        }

        Ok(())
    }

    fn time_in_force(&self) -> OrderTimeInForce {
        if let Some(tif) = self.tif {
            return tif;
        }

        match self.kind {
            Some(OrderKind::Market { .. })
            | Some(OrderKind::StopLossMarket { .. })
            | Some(OrderKind::TakeProfitMarket { .. }) => OrderTimeInForce::ImmediateOrCancel,
            _ => OrderTimeInForce::GoodTillTime,
        }
    }

    fn expiry(&self, signer: &SignerClient) -> Result<i64> {
        let time_in_force = self.time_in_force();
        let default = default_order_expiry(time_in_force, signer);
        Ok(self
            .expiry
            .map(|expiry| expiry.into_unix_millis())
            .unwrap_or(default))
    }

    fn nonce(&self) -> Option<i64> {
        self.nonce.map(Into::into)
    }

    fn api_key_override(&self) -> Option<i32> {
        self.api_key_override.map(Into::into)
    }

    fn resolved_client_order_id(&self) -> i64 {
        match self.client_order_id {
            ClientOrderIdStrategy::Manual(id) => id,
            ClientOrderIdStrategy::Auto => generate_client_order_id(),
        }
    }

    fn qty(&self) -> i64 {
        self.qty.expect("validated qty").into_inner()
    }

    fn side(&self) -> OrderSide {
        self.side.expect("validated side")
    }

    fn apply_price_offset(&self, price: Price, label: &'static str) -> Result<i32> {
        let base = price.into_ticks();
        let adjusted = base
            .checked_add(self.price_offset_ticks)
            .ok_or(Error::InvalidConfig {
                field: label,
                why: "price offset exceeds supported range",
            })?;
        Self::convert_price(label, adjusted)
    }

    fn convert_price(label: &'static str, value: i64) -> Result<i32> {
        i32::try_from(value).map_err(|_| Error::InvalidConfig {
            field: label,
            why: "price exceeds supported range",
        })
    }

    fn trigger_ticks(trigger: Price) -> Result<i32> {
        Self::convert_price("trigger", trigger.into_ticks())
    }

    fn market_price_ticks(&self, price: Option<Price>) -> Result<i32> {
        match price {
            Some(value) => Self::convert_price("avg_execution_price", value.into_ticks()),
            None => Ok(DEFAULT_MARKET_PRICE),
        }
    }

    fn order_type(&self, signer: &SignerClient) -> i32 {
        match self.kind.clone().expect("validated kind") {
            OrderKind::Limit { .. } => signer.order_type_limit(),
            OrderKind::Market { .. } => signer.order_type_market(),
            OrderKind::StopLossMarket { .. } => signer.order_type_stop_loss(),
            OrderKind::StopLossLimit { .. } => signer.order_type_stop_loss_limit(),
            OrderKind::TakeProfitMarket { .. } => signer.order_type_take_profit(),
            OrderKind::TakeProfitLimit { .. } => signer.order_type_take_profit_limit(),
        }
    }

    fn time_in_force_code(&self, signer: &SignerClient) -> i32 {
        resolve_time_in_force(self.time_in_force(), signer)
    }

    async fn submit(self) -> Result<Submission<transactions::CreateOrder>> {
        self.validate()?;
        let signer = self.client.signer_ref()?;
        let client_order_id = self.resolved_client_order_id();
        let nonce = self.nonce();
        let api_key = self.api_key_override();

        match self.kind.clone().expect("validated kind") {
            OrderKind::Limit { price } => {
                let price = self.apply_price_offset(price, "price")?;
                let expiry = self.expiry(signer)?;
                let (tx, response) = signer
                    .create_order(
                        self.market.into(),
                        client_order_id,
                        self.qty(),
                        price,
                        self.side().is_ask(),
                        self.order_type(signer),
                        self.time_in_force_code(signer),
                        self.reduce_only,
                        DEFAULT_TRIGGER_PRICE,
                        expiry,
                        nonce,
                        api_key,
                    )
                    .await?;
                Ok(Submission::new(tx, response))
            }
            OrderKind::Market {
                avg_execution_price,
                slippage,
            } => {
                let is_ask = self.side().is_ask();
                let base_amount = self.qty();
                let reduce_only = self.reduce_only;
                match slippage {
                    Some(SlippageStrategy::Enforced {
                        max_slippage,
                        ideal_price,
                    }) => {
                        let (tx, response) = signer
                            .create_market_order_limited_slippage(
                                self.market.into(),
                                client_order_id,
                                base_amount,
                                max_slippage,
                                is_ask,
                                reduce_only,
                                nonce,
                                api_key,
                                ideal_price.map(|price| price.into_ticks()),
                            )
                            .await?;
                        Ok(Submission::new(tx, response))
                    }
                    Some(SlippageStrategy::CheckOnly {
                        max_slippage,
                        ideal_price,
                    }) => {
                        let (tx, response) = signer
                            .create_market_order_if_slippage(
                                self.market.into(),
                                client_order_id,
                                base_amount,
                                max_slippage,
                                is_ask,
                                reduce_only,
                                nonce,
                                api_key,
                                ideal_price.map(|price| price.into_ticks()),
                            )
                            .await?;
                        Ok(Submission::new(tx, response))
                    }
                    None => {
                        let price = self.market_price_ticks(avg_execution_price)?;
                        let expiry = self.expiry(signer)?;
                        let (tx, response) = signer
                            .create_order(
                                self.market.into(),
                                client_order_id,
                                base_amount,
                                price,
                                is_ask,
                                self.order_type(signer),
                                self.time_in_force_code(signer),
                                reduce_only,
                                DEFAULT_TRIGGER_PRICE,
                                expiry,
                                nonce,
                                api_key,
                            )
                            .await?;
                        Ok(Submission::new(tx, response))
                    }
                }
            }
            OrderKind::StopLossMarket { trigger } | OrderKind::TakeProfitMarket { trigger } => {
                let trigger = Self::trigger_ticks(trigger)?;
                let expiry = self.expiry(signer)?;
                let (tx, response) = signer
                    .create_order(
                        self.market.into(),
                        client_order_id,
                        self.qty(),
                        DEFAULT_MARKET_PRICE,
                        self.side().is_ask(),
                        self.order_type(signer),
                        self.time_in_force_code(signer),
                        self.reduce_only,
                        trigger,
                        expiry,
                        nonce,
                        api_key,
                    )
                    .await?;
                Ok(Submission::new(tx, response))
            }
            OrderKind::StopLossLimit {
                trigger,
                limit_price,
            }
            | OrderKind::TakeProfitLimit {
                trigger,
                limit_price,
            } => {
                let trigger = Self::trigger_ticks(trigger)?;
                let price = self.apply_price_offset(limit_price, "limit_price")?;
                let expiry = self.expiry(signer)?;
                let (tx, response) = signer
                    .create_order(
                        self.market.into(),
                        client_order_id,
                        self.qty(),
                        price,
                        self.side().is_ask(),
                        self.order_type(signer),
                        self.time_in_force_code(signer),
                        self.reduce_only,
                        trigger,
                        expiry,
                        nonce,
                        api_key,
                    )
                    .await?;
                Ok(Submission::new(tx, response))
            }
        }
    }

    async fn sign(self) -> Result<SignedPayload<transactions::CreateOrder>> {
        self.validate()?;
        let signer = self.client.signer_ref()?;
        let client_order_id = self.resolved_client_order_id();
        let nonce = self.nonce();
        let api_key = self.api_key_override();

        match self.kind.clone().expect("validated kind") {
            OrderKind::Limit { price } => {
                let price = self.apply_price_offset(price, "price")?;
                let trigger = DEFAULT_TRIGGER_PRICE;
                let expiry = self.expiry(signer)?;
                signer
                    .sign_create_order(
                        self.market.into(),
                        client_order_id,
                        self.qty(),
                        price,
                        self.side().is_ask(),
                        self.order_type(signer),
                        self.time_in_force_code(signer),
                        self.reduce_only,
                        trigger,
                        expiry,
                        nonce,
                        api_key,
                    )
                    .await
                    .map_err(Into::into)
            }
            OrderKind::Market {
                avg_execution_price,
                slippage,
            } => {
                let is_ask = self.side().is_ask();
                let base_amount = self.qty();
                let reduce_only = self.reduce_only;
                match slippage {
                    Some(SlippageStrategy::Enforced {
                        max_slippage,
                        ideal_price,
                    }) => signer
                        .sign_market_order_limited_slippage(
                            self.market.into(),
                            client_order_id,
                            base_amount,
                            max_slippage,
                            is_ask,
                            reduce_only,
                            nonce,
                            api_key,
                            ideal_price.map(|price| price.into_ticks()),
                        )
                        .await
                        .map_err(Into::into),
                    Some(SlippageStrategy::CheckOnly { .. }) => Err(Error::InvalidConfig {
                        field: "slippage",
                        why: "slippage check is unavailable when signing orders",
                    }),
                    None => {
                        let price = self.market_price_ticks(avg_execution_price)?;
                        let expiry = self.expiry(signer)?;
                        signer
                            .sign_create_order(
                                self.market.into(),
                                client_order_id,
                                base_amount,
                                price,
                                is_ask,
                                self.order_type(signer),
                                self.time_in_force_code(signer),
                                reduce_only,
                                DEFAULT_TRIGGER_PRICE,
                                expiry,
                                nonce,
                                api_key,
                            )
                            .await
                            .map_err(Into::into)
                    }
                }
            }
            OrderKind::StopLossMarket { trigger } | OrderKind::TakeProfitMarket { trigger } => {
                let trigger = Self::trigger_ticks(trigger)?;
                let expiry = self.expiry(signer)?;
                signer
                    .sign_create_order(
                        self.market.into(),
                        client_order_id,
                        self.qty(),
                        DEFAULT_MARKET_PRICE,
                        self.side().is_ask(),
                        self.order_type(signer),
                        self.time_in_force_code(signer),
                        self.reduce_only,
                        trigger,
                        expiry,
                        nonce,
                        api_key,
                    )
                    .await
                    .map_err(Into::into)
            }
            OrderKind::StopLossLimit {
                trigger,
                limit_price,
            }
            | OrderKind::TakeProfitLimit {
                trigger,
                limit_price,
            } => {
                let trigger = Self::trigger_ticks(trigger)?;
                let price = self.apply_price_offset(limit_price, "limit_price")?;
                let expiry = self.expiry(signer)?;
                signer
                    .sign_create_order(
                        self.market.into(),
                        client_order_id,
                        self.qty(),
                        price,
                        self.side().is_ask(),
                        self.order_type(signer),
                        self.time_in_force_code(signer),
                        self.reduce_only,
                        trigger,
                        expiry,
                        nonce,
                        api_key,
                    )
                    .await
                    .map_err(Into::into)
            }
        }
    }
}

/// Order builder typestates.
pub struct OrderStateInit;
pub struct OrderStateSide;
pub struct OrderStateQty;
pub struct OrderStateReady;

/// Fluent order builder that enforces required fields at compile time.
pub struct OrderBuilder<'a, S> {
    state: OrderBuilderState<'a>,
    _marker: PhantomData<S>,
}

impl<'a> OrderBuilder<'a, OrderStateInit> {
    fn new(client: &'a LighterClient, market: MarketId) -> Self {
        Self {
            state: OrderBuilderState::new(client, market),
            _marker: PhantomData,
        }
    }

    pub fn buy(mut self) -> OrderBuilder<'a, OrderStateSide> {
        self.state.with_side(OrderSide::Bid);
        OrderBuilder {
            state: self.state,
            _marker: PhantomData,
        }
    }

    pub fn sell(mut self) -> OrderBuilder<'a, OrderStateSide> {
        self.state.with_side(OrderSide::Ask);
        OrderBuilder {
            state: self.state,
            _marker: PhantomData,
        }
    }
}

impl<'a> OrderBuilder<'a, OrderStateSide> {
    pub fn qty(mut self, qty: BaseQty) -> OrderBuilder<'a, OrderStateQty> {
        self.state.with_qty(qty);
        OrderBuilder {
            state: self.state,
            _marker: PhantomData,
        }
    }
}

impl<'a, S> OrderBuilder<'a, S> {
    pub fn ioc(mut self) -> Self {
        self.state.tif = Some(OrderTimeInForce::ImmediateOrCancel);
        self
    }

    pub fn gtt(mut self) -> Self {
        self.state.tif = Some(OrderTimeInForce::GoodTillTime);
        self
    }

    pub fn post_only(mut self) -> Self {
        self.state.tif = Some(OrderTimeInForce::PostOnly);
        self
    }

    pub fn reduce_only(mut self) -> Self {
        self.state.reduce_only = true;
        self
    }

    pub fn expires_at(mut self, expiry: Expiry) -> Self {
        self.state.expiry = Some(expiry);
        self
    }

    pub fn with_nonce(mut self, nonce: Nonce) -> Self {
        self.state.nonce = Some(nonce);
        self
    }

    pub fn with_api_key(mut self, index: ApiKeyIndex) -> Self {
        self.state.api_key_override = Some(index);
        self
    }

    pub fn with_client_order_id(mut self, id: i64) -> Self {
        self.state.client_order_id = ClientOrderIdStrategy::Manual(id);
        self
    }

    pub fn auto_client_id(mut self) -> Self {
        self.state.client_order_id = ClientOrderIdStrategy::Auto;
        self
    }

    pub fn with_avg_execution_price(mut self, price: Price) -> Self {
        match self.state.kind {
            Some(OrderKind::Market {
                ref mut avg_execution_price,
                ..
            }) => {
                *avg_execution_price = Some(price);
            }
            _ => {
                self.state.avg_execution_override = Some(price);
            }
        }
        self
    }

    pub fn with_slippage(mut self, max_slippage: f64) -> Self {
        self.update_slippage(SlippageStrategy::Enforced {
            max_slippage,
            ideal_price: None,
        });
        self
    }

    pub fn with_slippage_at_price(mut self, max_slippage: f64, price: Price) -> Self {
        self.update_slippage(SlippageStrategy::Enforced {
            max_slippage,
            ideal_price: Some(price),
        });
        self
    }

    pub fn with_slippage_check(mut self, max_slippage: f64) -> Self {
        self.update_slippage(SlippageStrategy::CheckOnly {
            max_slippage,
            ideal_price: None,
        });
        self
    }

    pub fn with_slippage_check_at_price(mut self, max_slippage: f64, price: Price) -> Self {
        self.update_slippage(SlippageStrategy::CheckOnly {
            max_slippage,
            ideal_price: Some(price),
        });
        self
    }

    pub fn with_price_offset(mut self, ticks: i64) -> Self {
        self.state.price_offset_ticks = ticks;
        self
    }

    fn update_slippage(&mut self, strategy: SlippageStrategy) {
        if let Some(OrderKind::Market {
            ref mut slippage, ..
        }) = self.state.kind
        {
            *slippage = Some(strategy);
        } else {
            self.state.slippage = Some(strategy);
        }
    }
}

impl<'a> OrderBuilder<'a, OrderStateQty> {
    pub fn limit(mut self, price: Price) -> OrderBuilder<'a, OrderStateReady> {
        self.state.slippage = None;
        self.state.avg_execution_override = None;
        self.state.with_kind(OrderKind::Limit { price });
        OrderBuilder {
            state: self.state,
            _marker: PhantomData,
        }
    }

    pub fn market(mut self) -> OrderBuilder<'a, OrderStateReady> {
        let avg = self.state.avg_execution_override.take();
        let slippage = self.state.slippage.take();
        self.state.with_kind(OrderKind::Market {
            avg_execution_price: avg,
            slippage,
        });
        OrderBuilder {
            state: self.state,
            _marker: PhantomData,
        }
    }

    pub fn stop_loss(mut self, trigger: Price) -> OrderBuilder<'a, OrderStateReady> {
        self.state.slippage = None;
        self.state.avg_execution_override = None;
        self.state.with_kind(OrderKind::StopLossMarket { trigger });
        OrderBuilder {
            state: self.state,
            _marker: PhantomData,
        }
    }

    pub fn stop_loss_limit(
        mut self,
        trigger: Price,
        price: Price,
    ) -> OrderBuilder<'a, OrderStateReady> {
        self.state.with_kind(OrderKind::StopLossLimit {
            trigger,
            limit_price: price,
        });
        OrderBuilder {
            state: self.state,
            _marker: PhantomData,
        }
    }

    pub fn take_profit(mut self, trigger: Price) -> OrderBuilder<'a, OrderStateReady> {
        self.state
            .with_kind(OrderKind::TakeProfitMarket { trigger });
        OrderBuilder {
            state: self.state,
            _marker: PhantomData,
        }
    }

    pub fn take_profit_limit(
        mut self,
        trigger: Price,
        price: Price,
    ) -> OrderBuilder<'a, OrderStateReady> {
        self.state.with_kind(OrderKind::TakeProfitLimit {
            trigger,
            limit_price: price,
        });
        OrderBuilder {
            state: self.state,
            _marker: PhantomData,
        }
    }
}

impl<'a> OrderBuilder<'a, OrderStateReady> {
    pub async fn submit(self) -> Result<Submission<transactions::CreateOrder>> {
        self.state.submit().await
    }

    pub async fn sign(self) -> Result<SignedPayload<transactions::CreateOrder>> {
        self.state.sign().await
    }

    fn into_state(self) -> OrderBuilderState<'a> {
        self.state
    }
}

pub struct BracketSubmission {
    pub entry: Submission<transactions::CreateOrder>,
    pub legs: Vec<Submission<transactions::CreateOrder>>,
}

pub struct BracketSigned {
    pub entry: SignedPayload<transactions::CreateOrder>,
    pub legs: Vec<SignedPayload<transactions::CreateOrder>>,
}

pub struct BracketBuilder<'a> {
    entry: OrderBuilderState<'a>,
    take_profit: Option<OrderKind>,
    stop_loss: Option<OrderKind>,
}

impl<'a> BracketBuilder<'a> {
    fn new(entry: OrderBuilderState<'a>) -> Self {
        Self {
            entry,
            take_profit: None,
            stop_loss: None,
        }
    }

    pub fn take_profit(mut self, trigger: Price) -> Self {
        self.take_profit = Some(OrderKind::TakeProfitMarket { trigger });
        self
    }

    pub fn take_profit_limit(mut self, trigger: Price, price: Price) -> Self {
        self.take_profit = Some(OrderKind::TakeProfitLimit {
            trigger,
            limit_price: price,
        });
        self
    }

    pub fn stop_loss(mut self, trigger: Price) -> Self {
        self.stop_loss = Some(OrderKind::StopLossMarket { trigger });
        self
    }

    pub fn stop_loss_limit(mut self, trigger: Price, price: Price) -> Self {
        self.stop_loss = Some(OrderKind::StopLossLimit {
            trigger,
            limit_price: price,
        });
        self
    }

    pub async fn submit(self) -> Result<BracketSubmission> {
        let BracketBuilder {
            entry,
            take_profit,
            stop_loss,
        } = self;

        let client = entry.client;
        let market = entry.market;
        let qty = entry.qty;
        let tif = entry.tif;
        let expiry = entry.expiry;
        let api_key_override = entry.api_key_override;
        let side = entry.side;

        let entry_submission = entry.submit().await?;
        let mut legs = Vec::new();

        if let Some(kind) = take_profit {
            let leg_state = Self::build_leg_state_from(
                client,
                market,
                qty,
                tif,
                expiry,
                api_key_override,
                side,
                kind,
            );
            legs.push(leg_state.submit().await?);
        }

        if let Some(kind) = stop_loss {
            let leg_state = Self::build_leg_state_from(
                client,
                market,
                qty,
                tif,
                expiry,
                api_key_override,
                side,
                kind,
            );
            legs.push(leg_state.submit().await?);
        }

        if legs.is_empty() {
            return Err(Error::InvalidConfig {
                field: "bracket",
                why: "configure at least one protective order",
            });
        }

        Ok(BracketSubmission {
            entry: entry_submission,
            legs,
        })
    }

    pub async fn sign(self) -> Result<BracketSigned> {
        let BracketBuilder {
            entry,
            take_profit,
            stop_loss,
        } = self;

        let client = entry.client;
        let market = entry.market;
        let qty = entry.qty;
        let tif = entry.tif;
        let expiry = entry.expiry;
        let api_key_override = entry.api_key_override;
        let side = entry.side;

        let entry_signed = entry.sign().await?;
        let mut legs = Vec::new();

        if let Some(kind) = take_profit {
            let leg_state = Self::build_leg_state_from(
                client,
                market,
                qty,
                tif,
                expiry,
                api_key_override,
                side,
                kind,
            );
            legs.push(leg_state.sign().await?);
        }

        if let Some(kind) = stop_loss {
            let leg_state = Self::build_leg_state_from(
                client,
                market,
                qty,
                tif,
                expiry,
                api_key_override,
                side,
                kind,
            );
            legs.push(leg_state.sign().await?);
        }

        if legs.is_empty() {
            return Err(Error::InvalidConfig {
                field: "bracket",
                why: "configure at least one protective order",
            });
        }

        Ok(BracketSigned {
            entry: entry_signed,
            legs,
        })
    }

    fn build_leg_state_from(
        client: &'a LighterClient,
        market: MarketId,
        qty: Option<BaseQty>,
        tif: Option<OrderTimeInForce>,
        expiry: Option<Expiry>,
        api_key_override: Option<ApiKeyIndex>,
        side: Option<OrderSide>,
        kind: OrderKind,
    ) -> OrderBuilderState<'a> {
        let mut state = OrderBuilderState::new(client, market);
        state.client_order_id = ClientOrderIdStrategy::Auto;
        state.side = side.map(OrderSide::opposite);
        state.qty = qty;
        state.reduce_only = true;
        state.tif = tif;
        state.expiry = expiry;
        state.api_key_override = api_key_override;
        state.kind = Some(kind);
        state
    }
}

impl<'a> OrderBuilder<'a, OrderStateReady> {
    pub fn bracket(self) -> BracketBuilder<'a> {
        BracketBuilder::new(self.into_state())
    }
}

#[derive(Default)]
pub struct OrderBatchBuilder<'a> {
    orders: Vec<OrderBuilderState<'a>>,
}

impl<'a> OrderBatchBuilder<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(mut self, builder: OrderBuilder<'a, OrderStateReady>) -> Self {
        self.orders.push(builder.into_state());
        self
    }

    pub fn add(&mut self, builder: OrderBuilder<'a, OrderStateReady>) -> &mut Self {
        self.orders.push(builder.into_state());
        self
    }

    pub async fn submit_all(self) -> Result<Vec<Submission<transactions::CreateOrder>>> {
        let mut results = Vec::with_capacity(self.orders.len());
        for order in self.orders {
            results.push(order.submit().await?);
        }
        Ok(results)
    }

    pub async fn sign_all(self) -> Result<Vec<SignedPayload<transactions::CreateOrder>>> {
        let mut results = Vec::with_capacity(self.orders.len());
        for order in self.orders {
            results.push(order.sign().await?);
        }
        Ok(results)
    }
}

#[derive(Debug, Clone)]
enum OrderKind {
    Limit {
        price: Price,
    },
    Market {
        avg_execution_price: Option<Price>,
        slippage: Option<SlippageStrategy>,
    },
    StopLossMarket {
        trigger: Price,
    },
    StopLossLimit {
        trigger: Price,
        limit_price: Price,
    },
    TakeProfitMarket {
        trigger: Price,
    },
    TakeProfitLimit {
        trigger: Price,
        limit_price: Price,
    },
}

/// Cancel order builder.
pub struct CancelOrderBuilder<'a> {
    client: &'a LighterClient,
    market: MarketId,
    order_index: i64,
    nonce: Option<Nonce>,
    api_key_override: Option<ApiKeyIndex>,
}

impl<'a> CancelOrderBuilder<'a> {
    fn new(client: &'a LighterClient, market: MarketId, order_index: i64) -> Self {
        Self {
            client,
            market,
            order_index,
            nonce: None,
            api_key_override: None,
        }
    }

    pub fn nonce(mut self, nonce: Nonce) -> Self {
        self.nonce = Some(nonce);
        self
    }

    pub fn api_key(mut self, index: ApiKeyIndex) -> Self {
        self.api_key_override = Some(index);
        self
    }

    pub async fn submit(self) -> Result<Submission<transactions::CancelOrder>> {
        let signer = self.client.signer_ref()?;
        let (payload, response) = signer
            .cancel_order(
                self.market.into(),
                self.order_index,
                self.nonce.map(Into::into),
                self.api_key_override.map(Into::into),
            )
            .await?;
        Ok(Submission::new(payload, response))
    }
}

/// Cancel all orders builder.
pub struct CancelAllBuilder<'a> {
    client: &'a LighterClient,
    tif: Option<OrderTimeInForce>,
    expiry: Option<Expiry>,
    nonce: Option<Nonce>,
    api_key_override: Option<ApiKeyIndex>,
}

impl<'a> CancelAllBuilder<'a> {
    fn new(client: &'a LighterClient) -> Self {
        Self {
            client,
            tif: None,
            expiry: None,
            nonce: None,
            api_key_override: None,
        }
    }

    pub fn tif(mut self, tif: OrderTimeInForce) -> Self {
        self.tif = Some(tif);
        self
    }

    pub fn expires_at(mut self, expiry: Expiry) -> Self {
        self.expiry = Some(expiry);
        self
    }

    pub fn nonce(mut self, nonce: Nonce) -> Self {
        self.nonce = Some(nonce);
        self
    }

    pub fn api_key(mut self, index: ApiKeyIndex) -> Self {
        self.api_key_override = Some(index);
        self
    }

    pub async fn submit(self) -> Result<Submission<String>> {
        let signer = self.client.signer_ref()?;
        let tif = resolve_time_in_force(self.tif.unwrap_or(OrderTimeInForce::GoodTillTime), signer);
        let expiry = self
            .expiry
            .map(|value| value.into_unix())
            .unwrap_or_else(|| {
                default_order_expiry(self.tif.unwrap_or(OrderTimeInForce::GoodTillTime), signer)
            });

        let (payload, response) = signer
            .cancel_all_orders(
                tif,
                expiry,
                self.nonce.map(Into::into),
                self.api_key_override.map(Into::into),
            )
            .await?;
        Ok(Submission::new(payload, response))
    }
}

/// Withdraw builder.
pub struct WithdrawBuilder<'a> {
    client: &'a LighterClient,
    amount: UsdcAmount,
    nonce: Option<Nonce>,
    api_key_override: Option<ApiKeyIndex>,
}

impl<'a> WithdrawBuilder<'a> {
    fn new(client: &'a LighterClient, amount: UsdcAmount) -> Self {
        Self {
            client,
            amount,
            nonce: None,
            api_key_override: None,
        }
    }

    pub fn nonce(mut self, nonce: Nonce) -> Self {
        self.nonce = Some(nonce);
        self
    }

    pub fn api_key(mut self, index: ApiKeyIndex) -> Self {
        self.api_key_override = Some(index);
        self
    }

    pub async fn submit(self) -> Result<Submission<transactions::Withdraw>> {
        let signer = self.client.signer_ref()?;
        let (payload, response) = signer
            .withdraw(
                self.amount.into(),
                self.nonce.map(Into::into),
                self.api_key_override.map(Into::into),
            )
            .await?;
        Ok(Submission::new(payload, response))
    }
}

const DEFAULT_TRIGGER_PRICE: i32 = 0;
const DEFAULT_MARKET_PRICE: i32 = -1;
const DEFAULT_ORDER_EXPIRY: i64 = -1;

fn resolve_time_in_force(time_in_force: OrderTimeInForce, signer: &SignerClient) -> i32 {
    match time_in_force {
        OrderTimeInForce::ImmediateOrCancel => signer.order_time_in_force_immediate_or_cancel(),
        OrderTimeInForce::GoodTillTime => signer.order_time_in_force_good_till_time(),
        OrderTimeInForce::PostOnly => signer.order_time_in_force_post_only(),
    }
}

fn default_order_expiry(time_in_force: OrderTimeInForce, signer: &SignerClient) -> i64 {
    match time_in_force {
        OrderTimeInForce::ImmediateOrCancel => signer.default_ioc_expiry(),
        OrderTimeInForce::GoodTillTime | OrderTimeInForce::PostOnly => DEFAULT_ORDER_EXPIRY,
    }
}
