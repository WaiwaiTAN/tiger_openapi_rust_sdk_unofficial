# Tiger Market Data

A read-only research data service built on Tiger Brokers' **official**
[`tigeropen`](https://github.com/tigerfintech/openapi-rust-sdk) Rust SDK.

The project downloads historical intraday JSONL for training/backtesting and
exposes normalized cached REST data plus live JSON quotes for model simulation.
Signing, tokens, REST transport, and Tiger's push protocol are handled by the
official SDK. This repository contains no trading routes.

## Credentials

The official SDK reads these variables directly:

```text
TIGEROPEN_TIGER_ID
TIGEROPEN_PRIVATE_KEY
TIGEROPEN_ACCOUNT
TIGEROPEN_TOKEN
TIGEROPEN_TOKEN_FILE
```

`TIGEROPEN_TIGER_ID` and `TIGEROPEN_PRIVATE_KEY` are required. Account and token
values depend on your Tiger account type. Do not commit credentials. An optional
properties fallback can be set as `tiger.credential_file` in the gateway config;
environment variables have higher priority.

## Historical downloader

The downloader uses Tiger's official trading-calendar and historical-timeline
APIs, writing one JSON line per trading day:

```bash
cargo build --release --bin downloader
./target/release/downloader \
  --symbols 7709.HK,000660.KS \
  --start 2025-01-01 \
  --end 2026-07-17 \
  --output-dir ../stock_data
```

It detaches by default on Unix and writes `downloader.log` in the output
directory. Add `--foreground` for an attached run.

## REST and live gateway

```bash
cp config/gateway.example.toml config/gateway.toml
cargo run --release --bin market_gateway -- --config config/gateway.toml
```

```bash
curl 'http://127.0.0.1:8765/health'
curl 'http://127.0.0.1:8765/v1/bars?symbol=7709.HK&interval=1d&start=2025-01-01&end=2026-07-17'
curl 'http://127.0.0.1:8765/v1/calendar?market=HK&start=2026-07-01&end=2026-07-17'
curl 'http://127.0.0.1:8765/v1/quote?symbols=7709.HK,AAPL'
websocat 'ws://127.0.0.1:8765/v1/stream?symbols=7709.HK,AAPL'
```

`/v1/stream` is the model-facing WebSocket. It subscribes through Tiger's
official persistent push client and emits normalized JSON updates with both
source and local-receive timestamps. If upstream push is unavailable, REST and
cached history continue while `/v1/stream` returns `501`.

The gateway is loopback-only by default. A remote bind requires both
`--allow-remote` and `--api-token-source PATH`.

## Verification

```bash
cargo test --all-features
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

Offline tests use mock providers and temporary SQLite databases; credentials are
not required.
