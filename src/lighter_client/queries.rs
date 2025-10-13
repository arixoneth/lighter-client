use super::{
    errors::{Error, Result},
    params::{HistoryFilter, OrderFilter, PageCursor, SortDir, TimeRange, Timestamp, TradeSort},
};
use crate::types::MarketId;

#[derive(Clone, Debug)]
pub struct TradesQuery<'a> {
    sort: TradeSort<'a>,
    limit: i64,
    market: Option<MarketId>,
    order_index: Option<i64>,
    direction: Option<SortDir>,
    from: Option<Timestamp>,
    cursor: Option<PageCursor<'a>>,
    filter: OrderFilter,
}

impl<'a> TradesQuery<'a> {
    pub fn new(sort: TradeSort<'a>, limit: i64) -> Result<Self> {
        if limit <= 0 {
            return Err(Error::InvalidConfig {
                field: "limit",
                why: "must be positive",
            });
        }
        Ok(Self {
            sort,
            limit,
            market: None,
            order_index: None,
            direction: None,
            from: None,
            cursor: None,
            filter: OrderFilter::All,
        })
    }

    pub fn market(mut self, market: MarketId) -> Self {
        self.market = Some(market);
        self
    }

    pub fn order_index(mut self, order_index: i64) -> Self {
        self.order_index = Some(order_index);
        self
    }

    pub fn direction(mut self, direction: SortDir) -> Self {
        self.direction = Some(direction);
        self
    }

    pub fn from(mut self, timestamp: Timestamp) -> Self {
        self.from = Some(timestamp);
        self
    }

    pub fn cursor(mut self, cursor: PageCursor<'a>) -> Self {
        self.cursor = Some(cursor);
        self
    }

    pub fn filter(mut self, filter: OrderFilter) -> Self {
        self.filter = filter;
        self
    }

    pub fn limit(&self) -> i64 {
        self.limit
    }

    pub fn sort(&self) -> &TradeSort<'a> {
        &self.sort
    }

    pub fn market_id(&self) -> Option<MarketId> {
        self.market
    }

    pub fn order_index_value(&self) -> Option<i64> {
        self.order_index
    }

    pub fn sort_direction(&self) -> Option<SortDir> {
        self.direction
    }

    pub fn from_timestamp(&self) -> Option<Timestamp> {
        self.from
    }

    pub fn cursor_ref(&self) -> Option<&PageCursor<'a>> {
        self.cursor.as_ref()
    }

    pub fn order_filter(&self) -> OrderFilter {
        self.filter
    }
}

#[derive(Clone, Debug)]
pub struct InactiveOrdersQuery<'a> {
    limit: i64,
    market: Option<MarketId>,
    filter: OrderFilter,
    between: Option<TimeRange>,
    cursor: Option<PageCursor<'a>>,
}

impl<'a> InactiveOrdersQuery<'a> {
    pub fn new(limit: i64) -> Result<Self> {
        if limit <= 0 {
            return Err(Error::InvalidConfig {
                field: "limit",
                why: "must be positive",
            });
        }

        Ok(Self {
            limit,
            market: None,
            filter: OrderFilter::All,
            between: None,
            cursor: None,
        })
    }

    pub fn market(mut self, market: MarketId) -> Self {
        self.market = Some(market);
        self
    }

    pub fn filter(mut self, filter: OrderFilter) -> Self {
        self.filter = filter;
        self
    }

    pub fn between(mut self, range: TimeRange) -> Self {
        self.between = Some(range);
        self
    }

    pub fn cursor(mut self, cursor: PageCursor<'a>) -> Self {
        self.cursor = Some(cursor);
        self
    }

    pub fn limit(&self) -> i64 {
        self.limit
    }

    pub fn market_id(&self) -> Option<MarketId> {
        self.market
    }

    pub fn order_filter(&self) -> OrderFilter {
        self.filter
    }

    pub fn between_range(&self) -> Option<TimeRange> {
        self.between
    }

    pub fn cursor_ref(&self) -> Option<&PageCursor<'a>> {
        self.cursor.as_ref()
    }
}

#[derive(Clone, Debug, Default)]
pub struct HistoryQuery<'a> {
    cursor: Option<PageCursor<'a>>,
    filter: Option<HistoryFilter<'a>>,
}

impl<'a> HistoryQuery<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cursor(mut self, cursor: PageCursor<'a>) -> Self {
        self.cursor = Some(cursor);
        self
    }

    pub fn filter(mut self, filter: HistoryFilter<'a>) -> Self {
        self.filter = Some(filter);
        self
    }

    pub fn cursor_ref(&self) -> Option<&PageCursor<'a>> {
        self.cursor.as_ref()
    }

    pub fn filter_ref(&self) -> Option<&HistoryFilter<'a>> {
        self.filter.as_ref()
    }
}
