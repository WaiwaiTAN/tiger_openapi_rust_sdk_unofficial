use super::{error::GatewayError, models::*, state::AppState};
use axum::{
    Json, Router,
    extract::{
        Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::Response,
    routing::get,
};
use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use serde::Deserialize;
use serde_json::{Value, json};
use tower::{ServiceBuilder, limit::ConcurrencyLimitLayer};
use tower_http::{
    limit::RequestBodyLimitLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use uuid::Uuid;
pub const ROUTES: &[&str] = &[
    "/health",
    "/ready",
    "/v1/bars",
    "/v1/calendar",
    "/v1/quote",
    "/v1/stream",
    "/v1/providers",
    "/openapi.json",
];
pub fn router(state: AppState) -> Router {
    let timeout = state.config.timeout();
    Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/v1/bars", get(bars))
        .route("/v1/calendar", get(calendar))
        .route("/v1/quote", get(quote))
        .route("/v1/stream", get(stream))
        .route("/v1/providers", get(providers))
        .route("/openapi.json", get(openapi))
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .layer(SetRequestIdLayer::new(
                    axum::http::HeaderName::from_static("x-request-id"),
                    MakeRequestUuid,
                ))
                .layer(PropagateRequestIdLayer::new(
                    axum::http::HeaderName::from_static("x-request-id"),
                ))
                .layer(RequestBodyLimitLayer::new(16 * 1024))
                .layer(ConcurrencyLimitLayer::new(64))
                .layer(TraceLayer::new_for_http().make_span_with(|request: &axum::http::Request<_>| {
                    tracing::info_span!("http_request", method=%request.method(), path=%request.uri().path())
                }))
                .layer(TimeoutLayer::new(timeout)),
        )
}
async fn health() -> Json<Value> {
    Json(json!({"status":"ok"}))
}
async fn ready(State(s): State<AppState>) -> Result<Json<Value>, GatewayError> {
    let h = s.provider.health().await?;
    if !h.available || !s.cache.ready() {
        return Err(GatewayError::Cache);
    }
    Ok(Json(
        json!({"status":"ready","provider":s.provider.provider_name(),"cache":"ready"}),
    ))
}
#[derive(Deserialize)]
struct BarsQuery {
    symbol: Option<String>,
    interval: Option<String>,
    start: Option<String>,
    end: Option<String>,
    adjustment: Option<String>,
    limit: Option<usize>,
    page_token: Option<String>,
    #[serde(default)]
    refresh: bool,
}
fn datetime(v: &str, end: bool) -> Result<DateTime<Utc>, GatewayError> {
    if let Ok(v) = DateTime::parse_from_rfc3339(v) {
        return Ok(v.with_timezone(&Utc));
    }
    let d = NaiveDate::parse_from_str(v, "%Y-%m-%d")
        .map_err(|_| GatewayError::Validation("date must be ISO-8601".into()))?;
    let t = if end {
        d.and_hms_opt(23, 59, 59)
    } else {
        d.and_hms_opt(0, 0, 0)
    }
    .unwrap();
    Ok(Utc.from_utc_datetime(&t))
}
async fn bars(
    State(s): State<AppState>,
    Query(q): Query<BarsQuery>,
) -> Result<Json<BarsResponse>, GatewayError> {
    let symbol = q
        .symbol
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| GatewayError::Validation("symbol is required".into()))?;
    let interval = q
        .interval
        .ok_or_else(|| GatewayError::Validation("interval is required".into()))?;
    if ![
        "1m", "3m", "5m", "10m", "15m", "30m", "1h", "1d", "1w", "1mo",
    ]
    .contains(&interval.as_str())
    {
        return Err(GatewayError::Validation("unsupported interval".into()));
    }
    let start = datetime(
        q.start
            .as_deref()
            .ok_or_else(|| GatewayError::Validation("start is required".into()))?,
        false,
    )?;
    let end = datetime(
        q.end
            .as_deref()
            .ok_or_else(|| GatewayError::Validation("end is required".into()))?,
        true,
    )?;
    if start > end {
        return Err(GatewayError::Validation(
            "start must not be after end".into(),
        ));
    }
    if (end - start).num_days() > s.config.max_date_span_days {
        return Err(GatewayError::Validation(
            "requested date range is too large".into(),
        ));
    }
    let limit = q.limit.unwrap_or(s.config.max_bars_per_response);
    if limit == 0 || limit > s.config.max_bars_per_response {
        return Err(GatewayError::Validation(
            "limit exceeds configured maximum".into(),
        ));
    }
    if q.page_token.as_ref().is_some_and(|v| {
        v.len() > 512
            || !v
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "-_.=".contains(c))
    }) {
        return Err(GatewayError::Validation("malformed page token".into()));
    }
    let provider = s.provider.provider_name();
    let missing = if q.refresh || q.page_token.is_some() {
        vec![(start, end)]
    } else {
        s.cache
            .missing_ranges(
                provider,
                &symbol,
                &interval,
                q.adjustment.as_deref(),
                start,
                end,
            )
            .map_err(|_| GatewayError::Cache)?
    };
    let mut next = None;
    let source_timestamp = Utc::now();
    for (missing_start, missing_end) in missing {
        let response = s
            .provider
            .bars(BarsRequest {
                symbol: symbol.clone(),
                interval: interval.clone(),
                start: missing_start,
                end: missing_end,
                adjustment: q.adjustment.clone(),
                limit,
                page_token: q.page_token.clone(),
                refresh: q.refresh,
            })
            .await?;
        next = response.next_page_token;
        s.cache
            .put(
                provider,
                &symbol,
                &interval,
                q.adjustment.as_deref(),
                &response.bars,
            )
            .map_err(|_| GatewayError::Cache)?;
        if q.page_token.is_none() {
            s.cache
                .mark_coverage(
                    provider,
                    &symbol,
                    &interval,
                    q.adjustment.as_deref(),
                    missing_start,
                    missing_end,
                )
                .map_err(|_| GatewayError::Cache)?;
        }
    }
    let mut result = s
        .cache
        .get(
            provider,
            &symbol,
            &interval,
            q.adjustment.as_deref(),
            start,
            end,
        )
        .map_err(|_| GatewayError::Cache)?;
    result.sort_by_key(|b| b.timestamp);
    result.dedup_by_key(|b| b.timestamp);
    if result.len() > limit {
        result.truncate(limit);
    }
    Ok(Json(BarsResponse {
        request_id: Uuid::new_v4(),
        provider: provider.into(),
        symbol,
        interval,
        adjustment: q.adjustment,
        timezone: None,
        currency: None,
        start,
        end,
        bars: result,
        next_page_token: next,
        source_timestamp,
    }))
}
#[derive(Deserialize)]
struct CalendarQuery {
    market: String,
    start: String,
    end: String,
}
async fn calendar(
    State(s): State<AppState>,
    Query(q): Query<CalendarQuery>,
) -> Result<Json<CalendarResponse>, GatewayError> {
    let start = NaiveDate::parse_from_str(&q.start, "%Y-%m-%d")
        .map_err(|_| GatewayError::Validation("invalid start date".into()))?;
    let end = NaiveDate::parse_from_str(&q.end, "%Y-%m-%d")
        .map_err(|_| GatewayError::Validation("invalid end date".into()))?;
    if start > end {
        return Err(GatewayError::Validation(
            "start must not be after end".into(),
        ));
    }
    Ok(Json(
        s.provider
            .trading_calendar(CalendarRequest {
                market: q.market,
                start,
                end,
            })
            .await?,
    ))
}
#[derive(Deserialize)]
struct QuoteQuery {
    symbols: String,
}
async fn quote(
    State(s): State<AppState>,
    Query(q): Query<QuoteQuery>,
) -> Result<Json<QuoteResponse>, GatewayError> {
    let symbols = q
        .symbols
        .split(',')
        .map(str::trim)
        .filter(|symbol| !symbol.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if symbols.is_empty() || symbols.len() > 50 {
        return Err(GatewayError::Validation(
            "symbols must contain between 1 and 50 comma-separated values".into(),
        ));
    }
    Ok(Json(s.provider.quote(QuoteRequest { symbols }).await?))
}
async fn stream(
    ws: WebSocketUpgrade,
    State(s): State<AppState>,
    Query(q): Query<QuoteQuery>,
) -> Result<Response, GatewayError> {
    let symbols = q
        .symbols
        .split(',')
        .map(str::trim)
        .filter(|symbol| !symbol.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if symbols.is_empty() || symbols.len() > 50 {
        return Err(GatewayError::Validation(
            "symbols must contain between 1 and 50 comma-separated values".into(),
        ));
    }
    let hub = s.realtime.ok_or(GatewayError::NotImplemented)?;
    let receiver = hub
        .subscribe(&symbols)
        .map_err(|_| GatewayError::Provider(super::error::ProviderError::Upstream))?;
    Ok(ws.on_upgrade(move |socket| stream_quotes(socket, receiver, symbols)))
}
async fn stream_quotes(
    mut socket: WebSocket,
    mut receiver: tokio::sync::broadcast::Receiver<super::realtime::RealtimeQuote>,
    symbols: Vec<String>,
) {
    while let Ok(quote) = receiver.recv().await {
        if !symbols.iter().any(|symbol| symbol == &quote.symbol) {
            continue;
        }
        let Ok(payload) = serde_json::to_string(&quote) else {
            continue;
        };
        if socket.send(Message::Text(payload.into())).await.is_err() {
            break;
        }
    }
}
async fn providers(State(s): State<AppState>) -> Json<Value> {
    Json(
        json!([{"name":s.provider.provider_name(),"configured":true,"available":true,"capabilities":["bars","calendar","quote"]}]),
    )
}
async fn openapi() -> Json<Value> {
    Json(
        json!({"openapi":"3.1.0","info":{"title":"Read-only Market Data Gateway","version":"1.0.0"},"paths":ROUTES.iter().map(|p|((*p).to_string(),json!({"get":{"responses":{"200":{"description":"Success"}}}}))).collect::<serde_json::Map<_,_>>() }),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::gateway::{
        cache::MarketCache, config::GatewayConfig, error::ProviderError,
        provider::MarketDataProvider, state::AppState,
    };
    use async_trait::async_trait;
    use axum::{body::Body, http::Request};
    use std::{net::SocketAddr, path::PathBuf, sync::Arc};
    use tower::ServiceExt;

    struct MockProvider;
    #[async_trait]
    impl MarketDataProvider for MockProvider {
        fn provider_name(&self) -> &'static str {
            "mock"
        }
        async fn health(&self) -> Result<ProviderHealth, ProviderError> {
            Ok(ProviderHealth { available: true })
        }
        async fn bars(&self, r: BarsRequest) -> Result<BarsResponse, ProviderError> {
            Ok(BarsResponse {
                request_id: Uuid::new_v4(),
                provider: "mock".into(),
                symbol: r.symbol,
                interval: r.interval,
                adjustment: r.adjustment,
                timezone: None,
                currency: None,
                start: r.start,
                end: r.end,
                bars: Vec::new(),
                next_page_token: None,
                source_timestamp: Utc::now(),
            })
        }
        async fn trading_calendar(
            &self,
            r: CalendarRequest,
        ) -> Result<CalendarResponse, ProviderError> {
            Ok(CalendarResponse {
                request_id: Uuid::new_v4(),
                provider: "mock".into(),
                market: r.market,
                days: Vec::new(),
            })
        }
    }
    fn app() -> Router {
        router(AppState {
            provider: Arc::new(MockProvider),
            cache: Arc::new(MarketCache::memory().unwrap()),
            config: GatewayConfig {
                bind: "127.0.0.1:8765".parse::<SocketAddr>().unwrap(),
                provider: "mock".into(),
                cache_path: PathBuf::from(":memory:"),
                request_timeout_seconds: 1,
                max_date_span_days: 30,
                max_bars_per_response: 100,
                acquire_quote_permission_on_startup: false,
                tiger: None,
            },
            realtime: None,
        })
    }
    #[test]
    fn no_trade_routes() {
        let banned = [
            "order", "trade", "position", "asset", "account", "transfer", "withdraw",
        ];
        for route in ROUTES {
            assert!(!banned.iter().any(|word| route.contains(word)));
        }
    }
    #[tokio::test]
    async fn health_works() {
        let response = app()
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
    }
    #[tokio::test]
    async fn bars_validation_is_offline() {
        for uri in [
            "/v1/bars?interval=1d&start=2026-01-01&end=2026-01-02",
            "/v1/bars?symbol=X&interval=2d&start=2026-01-01&end=2026-01-02",
            "/v1/bars?symbol=X&interval=1d&start=nope&end=2026-01-02",
            "/v1/bars?symbol=X&interval=1d&start=2026-02-01&end=2026-01-01",
            "/v1/bars?symbol=X&interval=1d&start=2025-01-01&end=2026-01-01",
            "/v1/bars?symbol=X&interval=1d&start=2026-01-01&end=2026-01-02&limit=101",
            "/v1/bars?symbol=X&interval=1d&start=2026-01-01&end=2026-01-02&page_token=%25",
        ] {
            let response = app()
                .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), 400, "{uri}");
        }
    }
}
