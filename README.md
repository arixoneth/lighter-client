# Lighter Client SDK (Rust)

The Lighter Client SDK provides a typed, Rust interface to the
[Lighter exchange](lighter.xyz). It wraps the generated
OpenAPI client with ergonomic builders, order-state typestates, websocket
helpers, and sensible defaults so you can build trading logic without dealing
with raw REST calls or signer plumbing.

## Highlights

- Builder-driven `LighterClient` with async/await and Tokio support
- Strongly typed identifiers (`MarketId`, `AccountId`, `ApiKeyIndex`, `Price`,
  `BaseQty`, etc.) to prevent unit mix ups
- DSL for constructing single orders, batches, and bracket strategies
- Handles for market data, funding, account management, and exports
- Websocket wrapper with automatic reconnect and typed events
- Bundled signer libraries for macOS (arm64) and Linux (x86_64) with optional
  overrides

## Installation

Add the crate to your project with Cargo:

```toml
[dependencies]
lighter_client = { git = "https://github.com/arixoneth/lighter-client" }
```

The crate targets Rust 1.70+ and requires Tokio because all high level APIs are
asynchronous.

## Quickstart

```rust
use lighter_client::{
    lighter_client::LighterClient,
    types::{AccountId, ApiKeyIndex},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = LighterClient::builder()
        .api_url("https://mainnet.zklighter.elliot.ai")
        .private_key(std::env::var("LIGHTER_PRIVATE_KEY")?)
        .account_index(AccountId::new(
            std::env::var("LIGHTER_ACCOUNT_INDEX")?.parse()?,
        ))
        .api_key_index(ApiKeyIndex::new(
            std::env::var("LIGHTER_API_KEY_INDEX")?.parse()?,
        ))
        .build()
        .await?;

    let account = client.account().details().await?;
    println!("Fetched {} sub account(s)", account.accounts.len());

    Ok(())
}
```

Set these environment variables before running:

- `LIGHTER_PRIVATE_KEY`
- `LIGHTER_ACCOUNT_INDEX`
- `LIGHTER_API_KEY_INDEX`
- optional `LIGHTER_API_URL`

## Signer libraries

The crate embeds the official signer shared libraries for macOS arm64 and Linux
x86_64. They are unpacked on demand to a temporary directory, so most users do
not need any extra configuration.

You can override the signer lookup using either:

- `LIGHTER_SIGNER_PATH` environment variable
- `LighterClient::builder().signer_library_path("path/to/lib")`

These overrides are useful when pinning to a custom signer build or when the
library lives outside the default search paths.

## Examples

The `examples/` directory contains focused scenarios:

- `examples/01_quickstart.rs` - configure the client and fetch account metadata
- `examples/02_order_lifecycle.rs` - create, sign, and optionally submit a limit order
- `examples/03_market_data.rs` - access public market data without credentials
- `examples/04_account_history.rs` - inspect trades, and withdrawals
- `examples/05_websocket.rs` - consume order book updates over websockets

Run them with `cargo run --example 01_quickstart` (replace the example name as
needed and export the required environment variables).

## Development

```bash
cargo fmt
cargo check
```

Files under `src/models/` and `src/apis/` are generated from the OpenAPI
specification; avoid editing them manually.
