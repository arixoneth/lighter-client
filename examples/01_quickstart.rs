//! Demonstrates how to configure the SDK and fetch basic account metadata.
//!
//! Required environment variables:
//! - `LIGHTER_PRIVATE_KEY`
//! - `LIGHTER_ACCOUNT_INDEX`
//! - `LIGHTER_API_KEY_INDEX`
//! - optional `LIGHTER_API_URL` (defaults to mainnet)

use lighter_client::{
    lighter_client::LighterClient,
    types::{AccountId, ApiKeyIndex},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_url = std::env::var("LIGHTER_API_URL")
        .unwrap_or_else(|_| "https://mainnet.zklighter.elliot.ai".to_string());
    let private_key = required_env("LIGHTER_PRIVATE_KEY");
    let account_index = required_env("LIGHTER_ACCOUNT_INDEX")
        .parse()
        .expect("LIGHTER_ACCOUNT_INDEX must be an integer");
    let api_key_index = required_env("LIGHTER_API_KEY_INDEX")
        .parse()
        .expect("LIGHTER_API_KEY_INDEX must be an integer");

    let client = LighterClient::builder()
        .api_url(api_url)
        .private_key(private_key)
        .account_index(AccountId::new(account_index))
        .api_key_index(ApiKeyIndex::new(api_key_index))
        .build()
        .await?;

    let details = client.account().details().await?;
    if let Some(account) = details.accounts.first() {
        let non_zero_positions: Vec<_> = account
            .positions
            .iter()
            .filter(|p| p.position_value.parse::<f64>().unwrap_or(0.0) > 0.0)
            .collect();
        println!(
            "Account {} has {} open position(s) and {} total order(s)",
            account.account_index,
            non_zero_positions.len(),
            account.total_order_count
        );
    } else {
        println!("No account details returned. Check your credentials.");
    }

    Ok(())
}

fn required_env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("set the {name} environment variable"))
}
