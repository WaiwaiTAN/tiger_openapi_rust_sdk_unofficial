# Provider development

Implement `MarketDataProvider` in a provider-specific module, translate provider payloads into normalized models there, and inject the implementation through `AppState`. Do not expose raw provider JSON, credentials, request signatures, or provider-specific account state.

Symbols are currently opaque provider-native strings. A future alias resolver may map forms such as `7709.HK` or `MU.US`, but it must be a separate explicit layer and must never silently rewrite ambiguous symbols.

Adapters should preserve timestamps and decimal precision, return unknown metadata as null, sort and deduplicate bars, classify retryable errors, and test against a mock HTTP server. Add capabilities to `/v1/providers` only after their upstream request and response contracts are verified.
