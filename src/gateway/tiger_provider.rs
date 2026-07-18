use super::{error::ProviderError, models::*, provider::MarketDataProvider};
use async_trait::async_trait;
use chrono::{NaiveDate, TimeZone, Utc};
use std::sync::Arc;
use tigeropen::{
    config::ClientConfig,
    model::quote_requests::{BriefRequest, KlineRequest, TradingCalendarRequest},
    quote::QuoteClient,
};
use tokio::sync::Semaphore;
use uuid::Uuid;
pub struct TigerMarketDataProvider {
    config: ClientConfig,
    concurrency: Arc<Semaphore>,
}
impl TigerMarketDataProvider {
    pub fn new(config: ClientConfig, max_concurrency: usize) -> Self {
        Self {
            config,
            concurrency: Arc::new(Semaphore::new(max_concurrency.max(1))),
        }
    }
    fn market(value: &str) -> Result<String, ProviderError> {
        match value.to_ascii_uppercase().as_str() {
            "US" | "HK" | "CN" | "SG" | "AU" | "ALL" => Ok(value.to_ascii_uppercase()),
            _ => Err(ProviderError::Unsupported),
        }
    }
    fn quote_client(&self) -> QuoteClient {
        QuoteClient::from_config(self.config.clone())
    }
}
#[async_trait]
impl MarketDataProvider for TigerMarketDataProvider {
    fn provider_name(&self) -> &'static str {
        "tiger"
    }
    async fn health(&self) -> Result<ProviderHealth, ProviderError> {
        Ok(ProviderHealth { available: true })
    }
    async fn bars(&self, r: BarsRequest) -> Result<BarsResponse, ProviderError> {
        let _permit = self
            .concurrency
            .acquire()
            .await
            .map_err(|_| ProviderError::Upstream)?;
        let period = match r.interval.as_str() {
            "1m" => "1min",
            "3m" => "3min",
            "5m" => "5min",
            "10m" => "10min",
            "15m" => "15min",
            "30m" => "30min",
            "1h" => "60min",
            "1d" => "day",
            "1w" => "week",
            "1mo" => "month",
            _ => return Err(ProviderError::Unsupported),
        };
        let right = match r.adjustment.as_deref() {
            None | Some("backward") => "br",
            Some("none") => "nr",
            _ => return Err(ProviderError::Unsupported),
        };
        let data = self
            .quote_client()
            .get_kline(KlineRequest {
                symbols: Some(vec![r.symbol.clone()]),
                period: Some(period.into()),
                right: Some(right.into()),
                begin_time: Some(r.start.timestamp_millis()),
                end_time: Some(r.end.timestamp_millis()),
                limit: Some(r.limit.min(i32::MAX as usize) as i32),
                page_token: r.page_token.clone(),
                ..Default::default()
            })
            .await
            .map_err(|_| ProviderError::Upstream)?;
        let mut bars = Vec::new();
        let next_page_token = data.iter().find_map(|line| {
            (!line.next_page_token.is_empty()).then(|| line.next_page_token.clone())
        });
        for line in data {
            for b in line.items {
                let timestamp = Utc
                    .timestamp_millis_opt(b.time)
                    .single()
                    .ok_or(ProviderError::Parse)?;
                bars.push(Bar {
                    timestamp,
                    open: rust_decimal::Decimal::from_f64_retain(b.open)
                        .ok_or(ProviderError::Parse)?,
                    high: rust_decimal::Decimal::from_f64_retain(b.high)
                        .ok_or(ProviderError::Parse)?,
                    low: rust_decimal::Decimal::from_f64_retain(b.low)
                        .ok_or(ProviderError::Parse)?,
                    close: rust_decimal::Decimal::from_f64_retain(b.close)
                        .ok_or(ProviderError::Parse)?,
                    volume: b.volume,
                    amount: rust_decimal::Decimal::from_f64_retain(b.amount),
                    is_complete: None,
                });
            }
        }
        bars.sort_by_key(|b| b.timestamp);
        bars.dedup_by_key(|b| b.timestamp);
        Ok(BarsResponse {
            request_id: Uuid::new_v4(),
            provider: "tiger".into(),
            symbol: r.symbol,
            interval: r.interval,
            adjustment: r.adjustment,
            timezone: None,
            currency: None,
            start: r.start,
            end: r.end,
            bars,
            next_page_token,
            source_timestamp: Utc::now(),
        })
    }
    async fn trading_calendar(
        &self,
        r: CalendarRequest,
    ) -> Result<CalendarResponse, ProviderError> {
        let _permit = self
            .concurrency
            .acquire()
            .await
            .map_err(|_| ProviderError::Upstream)?;
        let raw = self
            .quote_client()
            .get_trading_calendar(TradingCalendarRequest {
                market: Some(Self::market(&r.market)?),
                begin_date: Some(r.start.format("%Y-%m-%d").to_string()),
                end_date: Some(r.end.format("%Y-%m-%d").to_string()),
                ..Default::default()
            })
            .await
            .map_err(|_| ProviderError::Upstream)?;
        let mut days = Vec::new();
        for value in raw.into_iter().filter(|day| day.is_trading) {
            days.push(CalendarDay {
                date: NaiveDate::parse_from_str(&value.date, "%Y-%m-%d")
                    .map_err(|_| ProviderError::Parse)?,
                open: None,
                close: None,
            });
        }
        days.sort_by_key(|d| d.date);
        days.dedup_by_key(|d| d.date);
        Ok(CalendarResponse {
            request_id: Uuid::new_v4(),
            provider: "tiger".into(),
            market: r.market,
            days,
        })
    }
    async fn quote(&self, r: QuoteRequest) -> Result<QuoteResponse, ProviderError> {
        let values = self
            .quote_client()
            .get_real_time_quote(BriefRequest {
                symbols: Some(r.symbols),
                ..Default::default()
            })
            .await
            .map_err(|_| ProviderError::Upstream)?
            .into_iter()
            .map(|quote| {
                serde_json::json!({
                    "symbol": quote.symbol,
                    "latest_price": quote.latest_price,
                    "latest_time": quote.latest_time,
                    "open": quote.open,
                    "high": quote.high,
                    "low": quote.low,
                    "pre_close": quote.pre_close,
                    "volume": quote.volume,
                    "bid_price": quote.bid_price,
                    "bid_size": quote.bid_size,
                    "ask_price": quote.ask_price,
                    "ask_size": quote.ask_size,
                    "status": quote.status,
                    "change": quote.change,
                    "change_rate": quote.change_rate,
                })
            })
            .collect();
        Ok(QuoteResponse {
            request_id: Uuid::new_v4(),
            provider: "tiger".into(),
            quotes: values,
        })
    }
}
