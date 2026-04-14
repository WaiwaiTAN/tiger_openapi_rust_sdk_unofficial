# tiger_openapi_rust_sdk_unofficial

Unofficial Rust SDK for Tiger Brokers OpenAPI, with quote/trade client support and example utilities for downloading stock data with missing date detection.

## Features

- Rust library for Tiger Brokers OpenAPI interaction
- Quote client and trade client abstractions
- Client configuration loading from `.properties` files
- Download utility with batch fetching, retry handling, and missing-date analysis
- Example scripts for download workflow and K-line testing

## Repository Structure

- `Cargo.toml` — Rust package manifest
- `src/` — library and example application source code
  - `client_config.rs` — configuration loader and properties helper
  - `quote_client.rs` — quote API client logic
  - `trade_client.rs` — trade API client logic
  - `tiger_client.rs` — shared Tiger OpenAPI request helpers
  - `constants.rs` — API constants, default URLs, and default properties filenames
  - `properties.rs` — simple `.properties` parser
  - `main.rs` — downloader utility with missing-date detection logic
- `examples/` — usage examples
  - `downloader.rs` — stock data downloader with progress logging
  - `test_kline.rs` — K-line test/example client
- `properties/` — example production configuration files
- `properties_sandbox/` — example sandbox configuration files

## Getting Started

### Prerequisites

- Rust toolchain (stable)
- OpenSSL development libraries (`openssl` and headers) for the `openssl` crate

### Build

```bash
cargo build
```

### Run Examples

```bash
cargo run --example downloader
cargo run --example test_kline
```

> Note: Example commands assume the example files compile as Cargo examples. If the examples are not configured as Cargo examples, run them via `cargo run --bin <name>` or by editing the `examples/` directory to match your intended invocation.

## Configuration

This project uses property files for API credentials and environment configuration. Example files are provided in:

- `properties/tiger_openapi_config.properties`
- `properties/tiger_openapi_token.properties`
- `properties_sandbox/tiger_openapi_config.properties`
- `properties_sandbox/tiger_openapi_token.properties`

### Common properties

- `tiger_id`
- `account`
- `license`
- `private_key_pk1`
- `env` — set to `SANDBOX` for sandbox mode
- `token` — stored in the token properties file

### Default filenames

The SDK expects:

- `tiger_openapi_config.properties`
- `tiger_openapi_token.properties`

If you point `ClientConfig.props_path` at a directory, the SDK will load these filenames from that directory.

## Usage

As a library, import the crate and use the provided modules:

```rust
use tiger_openapi_rust_sdk_unofficial::client_config::ClientConfig;
use tiger_openapi_rust_sdk_unofficial::quote_client::QuoteClient;
use tiger_openapi_rust_sdk_unofficial::trade_client::TradeClient;
```

Create a `ClientConfig`, load the properties, and then instantiate the API clients.

## Notes

- The project is unofficial and intended for integration or experimentation with Tiger Brokers OpenAPI.
- It currently supports both production and sandbox endpoints.
- Missing-date detection and download progress tracking are implemented in `src/main.rs`.

## Dependencies

Key dependencies used by this project:

- `tokio` for async runtime
- `reqwest` for HTTP requests
- `serde` / `serde_json` for JSON handling
- `openssl`, `rsa`, `sha1`, `sha2` for request signing and encryption
- `chrono` for date handling
- `uuid` and `rand` for device/client identifiers

## License

This project is licensed under the MIT License. See the `LICENSE` file for details.
