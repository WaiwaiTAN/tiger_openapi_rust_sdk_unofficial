# Architecture

```mermaid
flowchart TD
  A[Research code / Codex-generated client] -->|localhost HTTP| B[Market-data gateway]
  B --> C[Normalized MarketDataProvider trait]
  C --> D[Tiger provider adapter]
  D --> E[Official tigeropen Rust SDK]
  E --> G[Tiger REST API]
  E --> H[Tiger TLS / Protobuf push]
  H --> I[Local JSON WebSocket]
  B --> F[(Normalized SQLite cache)]
```

The official `tigeropen` crate owns signing, token loading, HTTP transport,
response parsing, and the persistent real-time connection. The adapter translates
Tiger types into provider-neutral models. The gateway never constructs a trade
client and exposes no account or execution routes.

The cache records normalized bars and explicit fetched-range coverage. It does not infer missing holiday observations, currency, timezone, or incomplete-bar status.
