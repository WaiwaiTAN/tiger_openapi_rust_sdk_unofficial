# Unofficial Tiger Rust SDK and read-only market-data gateway

This repository contains four distinct surfaces:

- a reusable, unofficial Tiger quote SDK (`ClientConfig`, `TigerClient`, `QuoteClient`);
- a local, provider-neutral, read-only HTTP gateway (`market_gateway`);
- a legacy downloader binary (`downloader`);
- optional trading code, compiled only with the `trade` feature and never linked by the gateway.

The project is unofficial, is not endorsed by Tiger Brokers, and may lag upstream API changes. Validate data before financial or research use.

## Gateway architecture and API

The gateway binds to `127.0.0.1:8765` by default. It exposes only `GET /health`, `/ready`, `/v1/bars`, `/v1/calendar`, `/v1/quote`, `/v1/providers`, and `/openapi.json`. `/v1/quote` currently returns `501`: the existing SDK has no verified latest-quote request/response contract, and this implementation does not guess one. There are no account or execution routes.

Bars support `1m`, `3m`, `5m`, `10m`, `15m`, `30m`, `1h`, `1d`, `1w`, and `1mo`; Tiger adjustments map `backward` to `br` and `none` to `nr`. Symbols are provider-native and returned unchanged. Currency, timezone, and completion are null unless reliably known.

SQLite stores normalized UTC bars under `(provider, symbol, interval, adjustment, timestamp)`. Writes are transactional/upserted. Covered ranges avoid repeat requests; `refresh=true` bypasses coverage. Exchange holidays are neither forward-filled nor treated as errors.

See [architecture](docs/architecture.md), [security](docs/security.md), and [provider development](docs/provider-development.md).

## Build and test

```bash
cargo build --bin market_gateway
cargo test --all-features
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

Tests use mock providers, temporary SQLite databases, and fake values. They require no credentials or live network.

## Configure and run

Copy `config/gateway.example.toml` to `config/gateway.toml` and point `credential_directory` at an explicitly provisioned directory outside this repository containing:

```text
tiger_openapi_config.properties
tiger_openapi_token.properties
```

On Unix, credential files must not be group- or world-accessible (use mode `0600`). The gateway never searches for credentials and never defaults to legacy repository directories.

```bash
cargo run --bin market_gateway -- --config config/gateway.toml
curl 'http://127.0.0.1:8765/health'
curl 'http://127.0.0.1:8765/v1/bars?symbol=7709.HK&interval=1d&start=2025-10-16&end=2026-07-16'
curl 'http://127.0.0.1:8765/v1/calendar?market=HK&start=2026-07-01&end=2026-07-16'
cargo run --example gateway_client -- bars --symbol 7709.HK --interval 1d --start 2025-10-16 --end 2026-07-16
```

Non-loopback binding is refused unless both `--allow-remote` and `--api-token-source PATH` are set. The file supplies a bearer token; query-string authentication is unsupported. CORS is not enabled.

The downloader uses the library rather than compiling duplicate modules and requires an explicit `TIGER_CREDENTIAL_DIRECTORY`. It is a non-interactive CLI; the default target is the Hong Kong-listed SK Hynix 2× product:

```bash
TIGER_CREDENTIAL_DIRECTORY=/absolute/path/to/tiger-credentials \
  cargo run --bin downloader -- \
  --symbols 7709.HK \
  --start 2025-01-01 \
  --end 2026-07-17
```

Pass comma-separated symbols such as `--symbols 7709.HK,000660.KS`. The downloader runs detached by default, survives SSH disconnects, and writes data under `../stock_data/`. Build it once before starting a long-running download:

```bash
cargo build --release --bin downloader
TIGER_CREDENTIAL_DIRECTORY=/absolute/path/to/tiger-credentials \
  ./target/release/downloader \
  --symbols 7709.HK,000660.KS \
  --start 2025-01-01 \
  --end 2026-07-17

tail -f ../stock_data/downloader.log
```

The command prints the background PID before returning. Detached mode starts a separate Unix session and redirects standard input, output, and errors, so an SSH hangup does not terminate it. Pass `--foreground` to remain attached, `--output-dir PATH` to override the data directory, or `--log-file PATH` to override the detached log location. `--detach` remains accepted when you want to state the default explicitly. Build optional trading support with `--features trade`; it remains isolated from the gateway.
