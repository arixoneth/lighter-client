use std::{
    collections::HashMap,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use futures_util::{FutureExt, SinkExt, Stream, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};
use url::Url;

use crate::{
    errors::{WsClientError, WsResult},
    lighter_client::LighterClient,
    types::{AccountId, MarketId},
};

#[derive(Debug, Clone)]
pub struct ExponentialBackoff {
    pub initial: Duration,
    pub max: Duration,
    pub multiplier: f64,
}

impl Default for ExponentialBackoff {
    fn default() -> Self {
        Self {
            initial: Duration::from_millis(500),
            max: Duration::from_secs(30),
            multiplier: 2.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WsConfig {
    pub host: String,
    pub path: String,
    pub backoff: ExponentialBackoff,
}

impl Default for WsConfig {
    fn default() -> Self {
        Self {
            host: String::new(),
            path: "/stream".to_string(),
            backoff: ExponentialBackoff::default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SubscriptionSet {
    pub order_books: Vec<MarketId>,
    pub accounts: Vec<AccountId>,
}

impl SubscriptionSet {
    pub fn add_order_book(&mut self, market: MarketId) {
        self.order_books.push(market);
    }

    pub fn add_account(&mut self, account: AccountId) {
        self.accounts.push(account);
    }

    pub fn extend_order_books<I: IntoIterator<Item = MarketId>>(&mut self, markets: I) {
        self.order_books.extend(markets);
    }

    pub fn is_empty(&self) -> bool {
        self.order_books.is_empty() && self.accounts.is_empty()
    }
}

pub struct WsBuilder<'a> {
    client: &'a LighterClient,
    subscriptions: SubscriptionSet,
    config: WsConfig,
}

impl<'a> WsBuilder<'a> {
    pub(crate) fn new(client: &'a LighterClient) -> Self {
        let mut config = client.websocket_config().clone();
        if config.host.is_empty() {
            config.host = client.rest_base_path().to_string();
        }
        Self {
            client,
            subscriptions: SubscriptionSet::default(),
            config,
        }
    }

    pub fn subscribe_order_book(mut self, market: MarketId) -> Self {
        self.subscriptions.add_order_book(market);
        self
    }

    pub fn subscribe_order_books<I: IntoIterator<Item = MarketId>>(mut self, markets: I) -> Self {
        self.subscriptions.extend_order_books(markets);
        self
    }

    pub fn subscribe_account(mut self, account: AccountId) -> Self {
        self.subscriptions.add_account(account);
        self
    }

    pub fn backoff(mut self, backoff: ExponentialBackoff) -> Self {
        self.config.backoff = backoff;
        self
    }

    pub fn build(self) -> WsResult<WsClient> {
        WsClient::new(self.client, self.config, self.subscriptions)
    }

    pub async fn connect(self) -> WsResult<WsStream> {
        let client = self.build()?;
        let connection = client.connect().await?;
        Ok(WsStream::new(connection))
    }
}

#[derive(Debug, Clone)]
pub struct WsClient {
    config: WsConfig,
    subscriptions: SubscriptionSet,
    url: Url,
}

impl WsClient {
    fn new(
        client: &LighterClient,
        mut config: WsConfig,
        subscriptions: SubscriptionSet,
    ) -> WsResult<Self> {
        if subscriptions.is_empty() {
            return Err(WsClientError::EmptySubscriptions);
        }

        if config.host.is_empty() {
            config.host = client.rest_base_path().to_string();
        }

        let url = build_url(&config)?;
        Ok(Self {
            config,
            subscriptions,
            url,
        })
    }

    pub fn url(&self) -> &Url {
        &self.url
    }

    pub fn subscriptions(&self) -> &SubscriptionSet {
        &self.subscriptions
    }

    pub async fn connect(self) -> WsResult<WsConnection> {
        let (stream, _) = connect_async(self.url.as_str()).await?;
        Ok(WsConnection::new(
            self.url,
            stream,
            self.subscriptions,
            self.config.backoff,
        ))
    }
}

#[derive(Debug)]
pub struct WsConnection {
    url: Url,
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    subscriptions: SubscriptionSet,
    state: WsState,
    backoff: ExponentialBackoff,
}

#[derive(Debug, Default)]
struct WsState {
    order_books: HashMap<MarketId, OrderBookState>,
    accounts: HashMap<AccountId, AccountEvent>,
}

impl WsConnection {
    fn new(
        url: Url,
        stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
        subscriptions: SubscriptionSet,
        backoff: ExponentialBackoff,
    ) -> Self {
        Self {
            url,
            stream,
            subscriptions,
            state: WsState::default(),
            backoff,
        }
    }

    pub fn url(&self) -> &Url {
        &self.url
    }

    pub fn subscriptions(&self) -> &SubscriptionSet {
        &self.subscriptions
    }

    pub fn order_book_state(&self, market: MarketId) -> Option<&OrderBookState> {
        self.state.order_books.get(&market)
    }

    pub fn account_state(&self, account: AccountId) -> Option<&AccountEvent> {
        self.state.accounts.get(&account)
    }

    pub fn backoff(&self) -> &ExponentialBackoff {
        &self.backoff
    }

    pub async fn next_event(&mut self) -> WsResult<Option<WsEvent>> {
        while let Some(message) = self.stream.next().await {
            let message = message?;
            match message {
                Message::Text(text) => {
                    return self.handle_text_message(text).await;
                }
                Message::Binary(binary) => {
                    let text = String::from_utf8(binary).map_err(|_| {
                        WsClientError::InvalidMessage("invalid utf8 payload".to_string())
                    })?;
                    return self.handle_text_message(text).await;
                }
                Message::Ping(payload) => {
                    self.stream.send(Message::Pong(payload)).await?;
                }
                Message::Pong(_) => {
                    return Ok(Some(WsEvent::Pong));
                }
                Message::Close(frame) => {
                    let info = frame.map(|frame| CloseFrameInfo {
                        code: u16::from(frame.code),
                        reason: frame.reason.into_owned(),
                    });
                    return Ok(Some(WsEvent::Closed(info)));
                }
                Message::Frame(_) => {}
            }
        }

        Ok(None)
    }

    pub async fn close(mut self) -> WsResult<()> {
        self.stream.close(None).await?;
        Ok(())
    }

    async fn handle_text_message(&mut self, text: String) -> WsResult<Option<WsEvent>> {
        let message: Value = serde_json::from_str(&text)?;
        let message_type = message
            .get("type")
            .and_then(|value| value.as_str())
            .ok_or(WsClientError::MissingMessageType)?
            .to_owned();

        match message_type.as_str() {
            "connected" => {
                self.send_subscriptions().await?;
                Ok(Some(WsEvent::Connected))
            }
            "subscribed/order_book" => {
                let payload: OrderBookEnvelope = serde_json::from_value(message)?;
                let market = MarketId::from(parse_market_id(&payload.channel)?);
                let snapshot = OrderBookState::from_payload(payload.order_book);
                self.state.order_books.insert(market, snapshot.clone());
                Ok(Some(WsEvent::OrderBook(OrderBookEvent {
                    market,
                    state: snapshot,
                    delta: None,
                })))
            }
            "update/order_book" => {
                let payload: OrderBookEnvelope = serde_json::from_value(message)?;
                let market = MarketId::from(parse_market_id(&payload.channel)?);
                let delta = OrderBookDelta::from_payload(payload.order_book);
                let state = self
                    .state
                    .order_books
                    .entry(market)
                    .or_insert_with(|| OrderBookState::from_delta(&delta));
                state.apply_delta(&delta);
                Ok(Some(WsEvent::OrderBook(OrderBookEvent {
                    market,
                    state: state.clone(),
                    delta: Some(delta),
                })))
            }
            "subscribed/account_all" => {
                let event = self.handle_account_message(message, true)?;
                Ok(Some(WsEvent::Account(event)))
            }
            "update/account_all" => {
                let event = self.handle_account_message(message, false)?;
                Ok(Some(WsEvent::Account(event)))
            }
            _ => Ok(Some(WsEvent::Unknown(text))),
        }
    }

    async fn send_subscriptions(&mut self) -> WsResult<()> {
        for market in &self.subscriptions.order_books {
            let payload = json!({
                "type": "subscribe",
                "channel": format!("order_book/{}", market.into_inner()),
            })
            .to_string();
            self.stream.send(Message::Text(payload)).await?;
        }
        for account in &self.subscriptions.accounts {
            let payload = json!({
                "type": "subscribe",
                "channel": format!("account_all/{}", account.into_inner()),
            })
            .to_string();
            self.stream.send(Message::Text(payload)).await?;
        }
        Ok(())
    }

    fn handle_account_message(
        &mut self,
        mut message: Value,
        snapshot: bool,
    ) -> WsResult<AccountEventEnvelope> {
        let channel = message
            .get("channel")
            .and_then(|value| value.as_str())
            .ok_or_else(|| WsClientError::InvalidChannel("missing channel".to_string()))?;
        let account = AccountId::from(parse_account_index(channel)?);

        if let Some(obj) = message.as_object_mut() {
            obj.remove("type");
        }

        let event = AccountEvent::new(message.clone());
        self.state.accounts.insert(account, event.clone());

        Ok(AccountEventEnvelope {
            account,
            snapshot,
            event,
        })
    }
}

pub struct WsStream {
    connection: WsConnection,
}

impl WsStream {
    fn new(connection: WsConnection) -> Self {
        Self { connection }
    }

    pub fn connection(&self) -> &WsConnection {
        &self.connection
    }

    pub fn connection_mut(&mut self) -> &mut WsConnection {
        &mut self.connection
    }

    pub fn into_connection(self) -> WsConnection {
        self.connection
    }
}

impl Stream for WsStream {
    type Item = WsResult<WsEvent>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let fut = self.connection.next_event();
        futures_util::pin_mut!(fut);
        match futures_util::ready!(fut.poll_unpin(cx)) {
            Ok(Some(event)) => Poll::Ready(Some(Ok(event))),
            Ok(None) => Poll::Ready(None),
            Err(err) => Poll::Ready(Some(Err(err))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CloseFrameInfo {
    pub code: u16,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub enum WsEvent {
    Connected,
    Pong,
    OrderBook(OrderBookEvent),
    Account(AccountEventEnvelope),
    Closed(Option<CloseFrameInfo>),
    Unknown(String),
}

#[derive(Debug, Clone)]
pub struct OrderBookEvent {
    pub market: MarketId,
    pub state: OrderBookState,
    pub delta: Option<OrderBookDelta>,
}

#[derive(Debug, Clone)]
pub struct AccountEventEnvelope {
    pub account: AccountId,
    pub snapshot: bool,
    pub event: AccountEvent,
}

#[derive(Debug, Clone)]
pub struct AccountEvent(Value);

impl AccountEvent {
    pub fn new(value: Value) -> Self {
        Self(value)
    }

    pub fn into_inner(self) -> Value {
        self.0
    }

    pub fn as_value(&self) -> &Value {
        &self.0
    }
}

#[derive(Debug, Clone, Deserialize)]
struct OrderBookEnvelope {
    channel: String,
    #[serde(rename = "order_book")]
    order_book: OrderBookPayload,
}

#[derive(Debug, Clone, Deserialize)]
struct OrderBookPayload {
    asks: Vec<OrderBookLevel>,
    bids: Vec<OrderBookLevel>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrderBookLevel {
    pub price: String,
    #[serde(default)]
    pub size: String,
    #[serde(rename = "remaining_base_amount", default)]
    pub remaining_base_amount: Option<String>,
    #[serde(flatten, default)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub struct OrderBookDelta {
    pub asks: Vec<OrderBookLevel>,
    pub bids: Vec<OrderBookLevel>,
}

impl OrderBookDelta {
    fn from_payload(payload: OrderBookPayload) -> Self {
        Self {
            asks: payload.asks,
            bids: payload.bids,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OrderBookState {
    pub asks: Vec<OrderBookLevel>,
    pub bids: Vec<OrderBookLevel>,
}

impl OrderBookState {
    fn from_payload(payload: OrderBookPayload) -> Self {
        Self {
            asks: payload.asks,
            bids: payload.bids,
        }
    }

    fn from_delta(delta: &OrderBookDelta) -> Self {
        Self {
            asks: delta.asks.clone(),
            bids: delta.bids.clone(),
        }
    }

    fn apply_delta(&mut self, delta: &OrderBookDelta) {
        update_side(&mut self.asks, &delta.asks);
        update_side(&mut self.bids, &delta.bids);
    }
}

fn update_side(levels: &mut Vec<OrderBookLevel>, updates: &[OrderBookLevel]) {
    for update in updates {
        if let Some(existing) = levels.iter_mut().find(|level| level.price == update.price) {
            *existing = update.clone();
        } else {
            levels.push(update.clone());
        }
    }
    levels.retain(|level| !level_is_zero(level));
}

fn level_is_zero(level: &OrderBookLevel) -> bool {
    level
        .size
        .parse::<f64>()
        .map(|value| value == 0.0)
        .unwrap_or(false)
}

fn build_url(config: &WsConfig) -> WsResult<Url> {
    let mut candidate = config.host.clone();
    if candidate.starts_with("https://") {
        candidate = candidate.replacen("https://", "wss://", 1);
    } else if candidate.starts_with("http://") {
        candidate = candidate.replacen("http://", "ws://", 1);
    } else if !candidate.starts_with("ws://") && !candidate.starts_with("wss://") {
        candidate = format!("wss://{candidate}");
    }

    let mut url = Url::parse(&candidate)?;
    url.set_path(&config.path);
    Ok(url)
}

fn parse_market_id(channel: &str) -> WsResult<i32> {
    channel
        .split(|c| c == '/' || c == ':')
        .last()
        .and_then(|value| value.parse::<i32>().ok())
        .ok_or_else(|| WsClientError::InvalidChannel(channel.to_string()))
}

fn parse_account_index(channel: &str) -> WsResult<i64> {
    channel
        .split(|c| c == '/' || c == ':')
        .last()
        .and_then(|value| value.parse::<i64>().ok())
        .ok_or_else(|| WsClientError::InvalidChannel(channel.to_string()))
}
