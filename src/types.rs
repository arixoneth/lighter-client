use std::{fmt, num::NonZeroI64, ops::Deref};

use time::{Duration, OffsetDateTime};

/// Identifier for a market.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MarketId(pub i32);

impl MarketId {
    pub const fn new(value: i32) -> Self {
        Self(value)
    }

    pub const fn into_inner(self) -> i32 {
        self.0
    }
}

impl From<i32> for MarketId {
    fn from(value: i32) -> Self {
        Self::new(value)
    }
}

impl From<MarketId> for i32 {
    fn from(value: MarketId) -> Self {
        value.into_inner()
    }
}

impl fmt::Display for MarketId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Identifier for an account.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AccountId(pub i64);

impl AccountId {
    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    pub const fn into_inner(self) -> i64 {
        self.0
    }
}

impl From<i64> for AccountId {
    fn from(value: i64) -> Self {
        Self::new(value)
    }
}

impl From<AccountId> for i64 {
    fn from(value: AccountId) -> Self {
        value.into_inner()
    }
}

impl fmt::Display for AccountId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Wrapper around an API key index.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ApiKeyIndex(pub i32);

impl ApiKeyIndex {
    pub const fn new(value: i32) -> Self {
        Self(value)
    }

    pub const fn into_inner(self) -> i32 {
        self.0
    }
}

impl From<i32> for ApiKeyIndex {
    fn from(value: i32) -> Self {
        Self::new(value)
    }
}

impl From<ApiKeyIndex> for i32 {
    fn from(value: ApiKeyIndex) -> Self {
        value.into_inner()
    }
}

/// Representation of a price using integer ticks.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Price(i64);

impl Price {
    pub const fn ticks(value: i64) -> Self {
        Self(value)
    }

    pub const fn into_ticks(self) -> i64 {
        self.0
    }
}

impl From<i64> for Price {
    fn from(value: i64) -> Self {
        Self::ticks(value)
    }
}

impl From<Price> for i64 {
    fn from(value: Price) -> Self {
        value.into_ticks()
    }
}

/// Representation of a base quantity that must be strictly positive.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BaseQty(NonZeroI64);

impl BaseQty {
    pub fn new(value: NonZeroI64) -> Self {
        Self(value)
    }

    pub fn from_i64(value: i64) -> Option<Self> {
        NonZeroI64::new(value).map(Self)
    }

    pub const fn into_inner(self) -> i64 {
        self.0.get()
    }
}

impl TryFrom<i64> for BaseQty {
    type Error = &'static str;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        BaseQty::from_i64(value).ok_or("quantity must be non-zero")
    }
}

impl Deref for BaseQty {
    type Target = NonZeroI64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Representation of a nonce.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Nonce(i64);

impl Nonce {
    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    pub const fn into_inner(self) -> i64 {
        self.0
    }
}

impl From<i64> for Nonce {
    fn from(value: i64) -> Self {
        Self::new(value)
    }
}

impl From<Nonce> for i64 {
    fn from(value: Nonce) -> Self {
        value.into_inner()
    }
}

/// Representation of an expiry using [`OffsetDateTime`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Expiry(OffsetDateTime);

impl Expiry {
    pub fn unix(timestamp: i64) -> Self {
        let datetime = OffsetDateTime::from_unix_timestamp(timestamp)
            .expect("expiry unix timestamp should be representable");
        Self(datetime)
    }

    pub fn from_now(duration: Duration) -> Self {
        Self(OffsetDateTime::now_utc() + duration)
    }

    pub fn into_unix(self) -> i64 {
        self.0.unix_timestamp()
    }

    pub fn into_unix_millis(self) -> i64 {
        let seconds = self.0.unix_timestamp();
        let millis = (self.0.nanosecond() / 1_000_000) as i64;
        seconds
            .checked_mul(1_000)
            .expect("expiry unix timestamp should be representable in milliseconds")
            + millis
    }

    pub fn as_datetime(self) -> OffsetDateTime {
        self.0
    }
}

impl From<OffsetDateTime> for Expiry {
    fn from(value: OffsetDateTime) -> Self {
        Self(value)
    }
}

impl From<Expiry> for OffsetDateTime {
    fn from(value: Expiry) -> Self {
        value.as_datetime()
    }
}

/// Representation of a withdrawable USDC amount.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct UsdcAmount(f64);

impl UsdcAmount {
    pub const fn new(amount: f64) -> Self {
        Self(amount)
    }

    pub const fn into_inner(self) -> f64 {
        self.0
    }
}

impl From<f64> for UsdcAmount {
    fn from(value: f64) -> Self {
        Self::new(value)
    }
}

impl From<UsdcAmount> for f64 {
    fn from(value: UsdcAmount) -> Self {
        value.into_inner()
    }
}
