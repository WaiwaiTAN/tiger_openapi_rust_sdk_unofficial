use super::{error::ProviderError, models::*, provider::MarketDataProvider};
use crate::{quote_client::QuoteClient, tiger_enums::Market};
use async_trait::async_trait;
use chrono::{NaiveDate, TimeZone, Utc};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Semaphore;
use uuid::Uuid;
pub struct TigerMarketDataProvider {
    quote_client: QuoteClient,
    concurrency: Arc<Semaphore>,
}
impl TigerMarketDataProvider {
    pub fn new(quote_client: QuoteClient, max_concurrency: usize) -> Self {
        Self {
            quote_client,
            concurrency: Arc::new(Semaphore::new(max_concurrency.max(1))),
        }
    }
    fn market(value: &str) -> Result<Market, ProviderError> {
        match value.to_ascii_uppercase().as_str() {
            "US" => Ok(Market::US),
            "HK" => Ok(Market::HK),
            "CN" => Ok(Market::CN),
            "SG" => Ok(Market::SG),
            "ALL" => Ok(Market::ALL),
            _ => Err(ProviderError::Unsupported),
        }
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
            .quote_client
            .get_kline(
                &json!([r.symbol]),
                Some(period),
                Some(r.start.timestamp_millis()),
                Some(r.end.timestamp_millis()),
                Some(r.limit as i32),
                Some(right),
                r.page_token.as_deref(),
            )
            .await
            .map_err(|_| ProviderError::Upstream)?;
        let mut bars = Vec::new();
        for line in data {
            for b in line.items {
                let Some(time) = b.time else { continue };
                let timestamp = Utc
                    .timestamp_millis_opt(time)
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
            next_page_token: None,
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
            .quote_client
            .get_trading_calendar(
                Self::market(&r.market)?,
                &r.start.format("%Y-%m-%d").to_string(),
                &r.end.format("%Y-%m-%d").to_string(),
            )
            .await
            .map_err(|_| ProviderError::Upstream)?;
        let array = raw.as_array().ok_or(ProviderError::Parse)?;
        let mut days = Vec::new();
        for value in array {
            let date = value
                .get("date")
                .and_then(|v| v.as_str())
                .ok_or(ProviderError::Parse)?;
            days.push(CalendarDay {
                date: NaiveDate::parse_from_str(date, "%Y-%m-%d")
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
}
