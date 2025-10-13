//! Explores authenticated history endpoints for orders, trades, and withdrawals.
//!
//! Required environment variables:
//! - `LIGHTER_PRIVATE_KEY`
//! - `LIGHTER_ACCOUNT_INDEX`
//! - `LIGHTER_API_KEY_INDEX`
//! Optional:
//! - `LIGHTER_API_URL`
//! - `LIGHTER_MARKET_ID`

use lighter_client::lighter_client::{
    HistoryFilter, HistoryQuery, LighterClient, SortDir,
    TradeSort, TradesQuery,
};
use lighter_client::types::{AccountId, ApiKeyIndex, MarketId};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_url = std::env::var("LIGHTER_API_URL")
        .unwrap_or_else(|_| "https://mainnet.zklighter.elliot.ai".to_string());
    let private_key = required_env("LIGHTER_PRIVATE_KEY");
    let account_index = required_env("LIGHTER_ACCOUNT_INDEX")
        .parse()
        .expect("LIGHTER_ACCOUNT_INDEX must be an integer");
    let api_key_index = ApiKeyIndex::new(
        required_env("LIGHTER_API_KEY_INDEX")
            .parse()
            .expect("LIGHTER_API_KEY_INDEX must be an integer"),
    );
    let market = MarketId::new(env_or_default("LIGHTER_MARKET_ID", 1));

    let client = LighterClient::builder()
        .api_url(api_url)
        .private_key(private_key)
        .account_index(AccountId::new(account_index))
        .api_key_index(api_key_index)
        .build()
        .await?;


    let trades = client
        .account()
        .trades(
            TradesQuery::new(TradeSort::Timestamp, 20)?
                .market(market)
                .direction(SortDir::Desc),
        )
        .await?;
    if let Some(fill) = trades.trades.first() {
        println!(
            "Retrieved {} recent fills; newest trade id: {}",
            trades.trades.len(),
            fill.trade_id
        );
    } else {
        println!("No fills returned for market {market}");
    }

    let withdraws = client
        .account()
        .withdraw_history(HistoryQuery::new().filter(HistoryFilter::All))
        .await?;
    println!(
        "You have {} completed withdrawal(s)",
        withdraws.withdraws.len()
    );

    let next_nonce = client.account().next_nonce(api_key_index).await?;
    println!(
        "Next available nonce for API key {} is {}",
        api_key_index.into_inner(),
        next_nonce.nonce
    );

    Ok(())
}

fn env_or_default<T: std::str::FromStr + Copy>(name: &str, default: T) -> T {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

fn required_env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("set the {name} environment variable"))
}
