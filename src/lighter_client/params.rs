use std::{borrow::Cow, convert::TryFrom, fmt};

use super::errors::{Error, Result};

/// Selector used by block and transaction endpoints.
#[derive(Clone, Copy, Debug)]
pub enum By {
    Height,
    Commitment,
    Hash,
    L1Hash,
}

impl By {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            By::Height => "height",
            By::Commitment => "commitment",
            By::Hash => "hash",
            By::L1Hash => "l1_tx_hash",
        }
    }
}

impl fmt::Display for By {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Sorting direction helper for list endpoints.
#[derive(Clone, Copy, Debug)]
pub enum SortDir {
    Asc,
    Desc,
}

impl fmt::Display for SortDir {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SortDir::Asc => f.write_str("asc"),
            SortDir::Desc => f.write_str("desc"),
        }
    }
}

/// Cursor wrapper used for pagination parameters.
#[derive(Clone, Debug)]
pub struct PageCursor<'a>(pub(crate) Cow<'a, str>);

impl<'a> PageCursor<'a> {
    pub fn new(value: impl Into<Cow<'a, str>>) -> Result<Self> {
        let value = value.into();
        if value.is_empty() {
            return Err(Error::InvalidConfig {
                field: "cursor",
                why: "cannot be empty",
            });
        }
        Ok(Self(value))
    }

    pub(crate) fn as_str(&self) -> &str {
        self.0.as_ref()
    }

    pub(crate) fn into_owned(self) -> String {
        self.0.into_owned()
    }
}

/// Unix timestamp validated to be positive.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp(i64);

impl Timestamp {
    pub fn new(unix_seconds: i64) -> Result<Self> {
        if unix_seconds <= 0 {
            return Err(Error::InvalidConfig {
                field: "timestamp",
                why: "must be positive unix seconds",
            });
        }
        Ok(Self(unix_seconds))
    }

    pub fn into_unix(self) -> i64 {
        self.0
    }
}

impl TryFrom<i64> for Timestamp {
    type Error = Error;

    fn try_from(value: i64) -> Result<Self> {
        Timestamp::new(value)
    }
}

/// Time range helper used by endpoints requiring start and end bounds.
#[derive(Clone, Copy, Debug)]
pub struct TimeRange {
    start: Timestamp,
    end: Timestamp,
}

impl TimeRange {
    pub fn new(start: Timestamp, end: Timestamp) -> Result<Self> {
        if end < start {
            return Err(Error::InvalidConfig {
                field: "time_range",
                why: "end must be after start",
            });
        }
        Ok(Self { start, end })
    }

    pub fn start(self) -> Timestamp {
        self.start
    }

    pub fn end(self) -> Timestamp {
        self.end
    }

    pub(crate) fn bounds(self) -> (i64, i64) {
        (self.start.into_unix(), self.end.into_unix())
    }

    pub(crate) fn as_query(self) -> String {
        let (start, end) = self.bounds();
        format!("{start},{end}")
    }
}

/// Resolution options used by PnL and statistics endpoints.
#[derive(Clone, Debug)]
pub enum PnlResolution<'a> {
    Hour,
    Day,
    Week,
    Month,
    Custom(Cow<'a, str>),
}

impl<'a> PnlResolution<'a> {
    pub(crate) fn as_str(&self) -> &str {
        match self {
            PnlResolution::Hour => "hour",
            PnlResolution::Day => "day",
            PnlResolution::Week => "week",
            PnlResolution::Month => "month",
            PnlResolution::Custom(value) => value.as_ref(),
        }
    }
}

/// Candle resolution helper used by market data endpoints.
#[derive(Clone, Debug)]
pub enum CandleResolution<'a> {
    OneMinute,
    FiveMinutes,
    FifteenMinutes,
    OneHour,
    FourHours,
    OneDay,
    OneWeek,
    Custom(Cow<'a, str>),
}

impl<'a> CandleResolution<'a> {
    pub(crate) fn as_str(&self) -> &str {
        match self {
            CandleResolution::OneMinute => "1m",
            CandleResolution::FiveMinutes => "5m",
            CandleResolution::FifteenMinutes => "15m",
            CandleResolution::OneHour => "1h",
            CandleResolution::FourHours => "4h",
            CandleResolution::OneDay => "1d",
            CandleResolution::OneWeek => "1w",
            CandleResolution::Custom(value) => value.as_ref(),
        }
    }
}

/// Sort keys accepted by trade history endpoints.
#[derive(Clone, Debug)]
pub enum TradeSort<'a> {
    BlockHeight,
    Timestamp,
    TradeId,
    Custom(Cow<'a, str>),
}

impl<'a> TradeSort<'a> {
    pub(crate) fn as_str(&self) -> &str {
        match self {
            TradeSort::BlockHeight => "block_height",
            TradeSort::Timestamp => "timestamp",
            TradeSort::TradeId => "trade_id",
            TradeSort::Custom(value) => value.as_ref(),
        }
    }
}

/// Order side filter used by inactive order and trade listings.
#[derive(Clone, Copy, Debug)]
pub enum OrderFilter {
    All,
    Bids,
    Asks,
    Custom(i32),
}

impl OrderFilter {
    pub(crate) fn as_query(self) -> Option<i32> {
        match self {
            OrderFilter::All => None,
            OrderFilter::Bids => Some(0),
            OrderFilter::Asks => Some(1),
            OrderFilter::Custom(value) => Some(value),
        }
    }
}

/// Side selector for funding history endpoints.
#[derive(Clone, Copy, Debug)]
pub enum FundingSide {
    Bid,
    Ask,
}

impl FundingSide {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            FundingSide::Bid => "bid",
            FundingSide::Ask => "ask",
        }
    }
}

/// Filter applied to public pool listings.
#[derive(Clone, Debug)]
pub enum PoolFilter<'a> {
    Active,
    Inactive,
    Custom(Cow<'a, str>),
}

impl<'a> PoolFilter<'a> {
    pub(crate) fn as_str(&self) -> &str {
        match self {
            PoolFilter::Active => "active",
            PoolFilter::Inactive => "inactive",
            PoolFilter::Custom(value) => value.as_ref(),
        }
    }
}

/// Filter applied to deposit/withdraw history endpoints.
#[derive(Clone, Debug)]
pub enum HistoryFilter<'a> {
    All,
    Pending,
    Claimable,
    Custom(Cow<'a, str>),
}

impl<'a> HistoryFilter<'a> {
    pub(crate) fn as_str(&self) -> &str {
        match self {
            HistoryFilter::All => "all",
            HistoryFilter::Pending => "pending",
            HistoryFilter::Claimable => "claimable",
            HistoryFilter::Custom(value) => value.as_ref(),
        }
    }
}
