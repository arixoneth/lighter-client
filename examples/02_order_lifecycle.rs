//! Shows how to prepare, sign, and optionally submit a limit order.
//!
//! Required environment variables:
//! - `LIGHTER_PRIVATE_KEY`
//! - `LIGHTER_ACCOUNT_INDEX`
//! - `LIGHTER_API_KEY_INDEX`
//! Optional overrides:
//! - `LIGHTER_API_URL`
//! - `LIGHTER_MARKET_ID`
//! - `LIGHTER_ORDER_QTY`
//! - `LIGHTER_LIMIT_TICKS`

use lighter_client::{
    lighter_client::{LighterClient, OrderBuilder, OrderStateReady},
    types::{AccountId, ApiKeyIndex, BaseQty, Expiry, MarketId, Price},
};
use time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = build_client().await?;
    let market = MarketId::new(env_or_default("LIGHTER_MARKET_ID", 0));
    let quantity = env_or_default("LIGHTER_ORDER_QTY", 1_00);
    let limit_ticks = env_or_default("LIGHTER_LIMIT_TICKS", 410_000);

    let signed = limit_order(&client, market, quantity, limit_ticks)
        .expires_at(Expiry::from_now(Duration::minutes(5)))
        .post_only()
        .sign()
        .await?;

    println!("Signed limit order payload: {}", signed.payload());

    // Uncomment to submit the live order:
    // let submission = limit_order(&client, market, quantity, limit_ticks)
    //     .expires_at(Expiry::from_now(Duration::minutes(5)))
    //     .post_only()
    //     .submit()
    //     .await?;
    // println!("Order accepted with tx hash {}", submission.response().tx_hash);

    Ok(())
}

fn limit_order<'a>(
    client: &'a LighterClient,
    market: MarketId,
    quantity: i64,
    limit_ticks: i64,
) -> OrderBuilder<'a, OrderStateReady> {
    let qty = BaseQty::try_from(quantity).expect("quantity must be non-zero");
    client
        .order(market)
        .buy()
        .qty(qty)
        .limit(Price::ticks(limit_ticks))
}

async fn build_client() -> Result<LighterClient, Box<dyn std::error::Error>> {
    let api_url = std::env::var("LIGHTER_API_URL")
        .unwrap_or_else(|_| "https://mainnet.zklighter.elliot.ai".to_string());
    let private_key = required_env("LIGHTER_PRIVATE_KEY");
    let account_index = required_env("LIGHTER_ACCOUNT_INDEX")
        .parse()
        .expect("LIGHTER_ACCOUNT_INDEX must be an integer");
    let api_key_index = required_env("LIGHTER_API_KEY_INDEX")
        .parse()
        .expect("LIGHTER_API_KEY_INDEX must be an integer");

    Ok(LighterClient::builder()
        .api_url(api_url)
        .private_key(private_key)
        .account_index(AccountId::new(account_index))
        .api_key_index(ApiKeyIndex::new(api_key_index))
        .build()
        .await?)
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
