//! Shows how to create grouped orders with OCO (One-Cancels-Other) relationship.
//!
//! This example demonstrates creating a take-profit and stop-loss pair where
//! executing one order automatically cancels the other.
//!
//! Two approaches are shown:
//! 1. High-level builder pattern (recommended)
//! 2. Low-level FFI interface (for advanced use cases)
//!
//! Required environment variables:
//! - `LIGHTER_PRIVATE_KEY`
//! - `LIGHTER_ACCOUNT_INDEX`
//! - `LIGHTER_API_KEY_INDEX`
//! Optional overrides:
//! - `LIGHTER_API_URL`
//! - `LIGHTER_MARKET_ID`
//! - `LIGHTER_ORDER_QTY`
//! - `LIGHTER_TP_TRIGGER_PRICE` (take-profit trigger price)
//! - `LIGHTER_TP_LIMIT_PRICE` (take-profit limit price)
//! - `LIGHTER_SL_TRIGGER_PRICE` (stop-loss trigger price)
//! - `LIGHTER_SL_LIMIT_PRICE` (stop-loss limit price)

use std::num::NonZeroI64;

use lighter_client::{
    lighter_client::LighterClient,
    types::{AccountId, ApiKeyIndex, BaseQty, CreateOrderTxReq, Expiry, MarketId, Price},
};
use time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = build_client().await?;

    // Configuration
    let market_id = MarketId::new(env_or_default("LIGHTER_MARKET_ID", 0));
    let quantity =
        BaseQty::new(NonZeroI64::new(env_or_default("LIGHTER_ORDER_QTY", 1_00)).unwrap());

    // Example: Take-profit at 420k, Stop-loss at 400k
    let expiry = Expiry::from_now(Duration::days(7));
    let entry_price = Price::ticks(410_000);
    let tp_trigger = Price::ticks(env_or_default("LIGHTER_TP_TRIGGER_PRICE", 420_000));
    let sl_trigger = Price::ticks(env_or_default("LIGHTER_SL_TRIGGER_PRICE", 400_000));

    println!("=== Approach 1: High-Level Builder Pattern (Recommended) ===\n");

    // Build OTOCO orders using fluent builder pattern
    client
        .grouped_orders()
        .otoco()
        .add_order(
            client
                .order(market_id)
                .buy()
                .qty(quantity)
                .expires_at(expiry)
                .limit(entry_price)
                .post_only(),
        )
        .add_order(
            client
                .order(market_id)
                .sell()
                .qty(quantity)
                .expires_at(expiry)
                .take_profit(tp_trigger)
                .reduce_only(),
        )
        .add_order(
            client
                .order(market_id)
                .sell()
                .qty(quantity)
                .expires_at(expiry)
                .stop_loss(sl_trigger)
                .reduce_only(),
        )
        .submit()
        .await?;

    println!("Orders submitted in OTOCO grouping.");
    println!("Grouping type: OTOCO (One-Triggers-Other-Cancels-Other)");
    println!("When one order executes, the other will be automatically cancelled.\n");

    // Uncomment to submit the live orders:
    // let submission = client
    //     .grouped_orders()
    //     .oco()
    //     .add_order(
    //         client
    //             .order(market_id)
    //             .sell()
    //             .qty(quantity)
    //             .take_profit_limit(tp_trigger, tp_limit)
    //             .reduce_only(),
    //     )
    //     .add_order(
    //         client
    //             .order(market_id)
    //             .sell()
    //             .qty(quantity)
    //             .stop_loss_limit(sl_trigger, sl_limit)
    //             .reduce_only(),
    //     )
    //     .submit()
    //     .await?;
    // println!("Orders accepted with tx hash: {}", submission.response().tx_hash);

    // Uncomment to use the low-level API:
    // println!("\n=== Approach 2: Low-Level FFI Interface ===\n");
    // Low-level approach using direct FFI interface
    // _demonstrate_low_level_api(
    //     &client, market_id, quantity, tp_trigger, tp_limit, sl_trigger, sl_limit,
    // )
    // .await?;

    Ok(())
}

async fn _demonstrate_low_level_api(
    client: &LighterClient,
    market_id: MarketId,
    _quantity: BaseQty, // Not used - SL/TP orders in grouped orders must have base_amount = 0
    tp_trigger: Price,
    tp_limit: Price,
    sl_trigger: Price,
    sl_limit: Price,
) -> Result<(), Box<dyn std::error::Error>> {
    let signer = client
        .signer()
        .expect("Client must be initialized with a signer");

    // Create take-profit order (sell when price goes up)
    // Note: base_amount is 0 for reduce-only SL/TP orders in grouped orders
    let take_profit_order = CreateOrderTxReq {
        market_index: market_id.into_inner() as u8,
        client_order_index: 0,
        base_amount: 0, // Must be 0 for reduce-only SL/TP in grouped orders
        price: tp_limit.into_ticks() as u32,
        is_ask: 1, // Sell
        order_type: signer.order_type_take_profit_limit() as u8,
        time_in_force: signer.order_time_in_force_good_till_time() as u8,
        reduce_only: 1, // Only reduce position
        trigger_price: tp_trigger.into_ticks() as u32,
        order_expiry: -1, // -1 means 28 days (default expiry)
    };

    // Create stop-loss order (sell when price goes down)
    // Note: base_amount is 0 for reduce-only SL/TP orders in grouped orders
    let stop_loss_order = CreateOrderTxReq {
        market_index: market_id.into_inner() as u8,
        client_order_index: 0,
        base_amount: 0, // Must be 0 for reduce-only SL/TP in grouped orders
        price: sl_limit.into_ticks() as u32,
        is_ask: 1, // Sell
        order_type: signer.order_type_stop_loss_limit() as u8,
        time_in_force: signer.order_time_in_force_good_till_time() as u8,
        reduce_only: 1, // Only reduce position
        trigger_price: sl_trigger.into_ticks() as u32,
        order_expiry: -1, // -1 means 28 days (default expiry)
    };

    let orders = vec![take_profit_order, stop_loss_order];

    // Sign the grouped orders with OCO relationship
    let signed = signer
        .sign_create_grouped_orders(
            signer.grouping_type_oco(),
            &orders,
            None, // nonce
            None, // api_key_index
        )
        .await?;

    println!("Low-level API signed payload:");
    println!("{}", signed.payload());
    println!("\nBoth approaches produce equivalent results.\n");

    Ok(())
}

async fn build_client() -> Result<LighterClient, Box<dyn std::error::Error>> {
    let api_url = std::env::var("LIGHTER_API_URL")
        .unwrap_or_else(|_| "https://testnet.zklighter.elliot.ai".to_string());
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
