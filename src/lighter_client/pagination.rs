use std::future::Future;

use futures_util::stream::{self, Stream};

use super::errors::Result;

pub(crate) trait Paginated {
    type Item;

    fn into_parts(self) -> (Vec<Self::Item>, Option<String>);
}

impl Paginated for crate::models::Trades {
    type Item = crate::models::Trade;

    fn into_parts(self) -> (Vec<Self::Item>, Option<String>) {
        (self.trades, self.next_cursor)
    }
}

impl Paginated for crate::models::Orders {
    type Item = crate::models::Order;

    fn into_parts(self) -> (Vec<Self::Item>, Option<String>) {
        (self.orders, self.next_cursor)
    }
}

impl Paginated for crate::models::LiquidationInfos {
    type Item = crate::models::Liquidation;

    fn into_parts(self) -> (Vec<Self::Item>, Option<String>) {
        (self.liquidations, self.next_cursor)
    }
}

impl Paginated for crate::models::PositionFundings {
    type Item = crate::models::PositionFunding;

    fn into_parts(self) -> (Vec<Self::Item>, Option<String>) {
        (self.position_fundings, self.next_cursor)
    }
}

impl Paginated for crate::models::DepositHistory {
    type Item = crate::models::DepositHistoryItem;

    fn into_parts(self) -> (Vec<Self::Item>, Option<String>) {
        let next = if self.cursor.is_empty() {
            None
        } else {
            Some(self.cursor)
        };
        (self.deposits, next)
    }
}

impl Paginated for crate::models::WithdrawHistory {
    type Item = crate::models::WithdrawHistoryItem;

    fn into_parts(self) -> (Vec<Self::Item>, Option<String>) {
        let next = if self.cursor.is_empty() {
            None
        } else {
            Some(self.cursor)
        };
        (self.withdraws, next)
    }
}

impl Paginated for crate::models::TransferHistory {
    type Item = crate::models::TransferHistoryItem;

    fn into_parts(self) -> (Vec<Self::Item>, Option<String>) {
        let next = if self.cursor.is_empty() {
            None
        } else {
            Some(self.cursor)
        };
        (self.transfers, next)
    }
}

pub(crate) fn paginate_items<'a, R, F, Fut>(
    initial_cursor: Option<String>,
    fetch: F,
) -> impl Stream<Item = Result<R::Item>> + 'a
where
    R: Paginated + 'a,
    R::Item: 'a,
    F: FnMut(Option<String>) -> Fut + 'a,
    Fut: Future<Output = Result<R>> + 'a,
{
    struct PaginationState<T, F> {
        cursor: Option<String>,
        buffer: Vec<T>,
        finished: bool,
        fetch: F,
    }

    stream::try_unfold(
        PaginationState {
            cursor: initial_cursor,
            buffer: Vec::new(),
            finished: false,
            fetch,
        },
        move |mut state| async move {
            loop {
                if let Some(item) = state.buffer.pop() {
                    return Ok(Some((item, state)));
                }

                if state.finished {
                    return Ok(None);
                }

                let response = {
                    let fetch = &mut state.fetch;
                    fetch(state.cursor.clone()).await?
                };
                let (mut items, next_cursor) = response.into_parts();
                state.cursor = next_cursor;
                if items.is_empty() {
                    if state.cursor.is_none() {
                        state.finished = true;
                        return Ok(None);
                    }
                    continue;
                }

                items.reverse();
                state.buffer = items;
            }
        },
    )
}
