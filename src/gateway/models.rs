use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Bar {
    pub timestamp: DateTime<Utc>,
    #[serde(with = "rust_decimal::serde::str")]
    pub open: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub high: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub low: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub close: Decimal,
    pub volume: i64,
    #[serde(with = "rust_decimal::serde::str_option")]
    pub amount: Option<Decimal>,
    pub is_complete: Option<bool>,
}
#[derive(Debug, Clone)]
pub struct BarsRequest {
    pub symbol: String,
    pub interval: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub adjustment: Option<String>,
    pub limit: usize,
    pub page_token: Option<String>,
    pub refresh: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarsResponse {
    pub request_id: Uuid,
    pub provider: String,
    pub symbol: String,
    pub interval: String,
    pub adjustment: Option<String>,
    pub timezone: Option<String>,
    pub currency: Option<String>,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub bars: Vec<Bar>,
    pub next_page_token: Option<String>,
    pub source_timestamp: DateTime<Utc>,
}
#[derive(Debug, Clone)]
pub struct CalendarRequest {
    pub market: String,
    pub start: NaiveDate,
    pub end: NaiveDate,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarDay {
    pub date: NaiveDate,
    pub open: Option<String>,
    pub close: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarResponse {
    pub request_id: Uuid,
    pub provider: String,
    pub market: String,
    pub days: Vec<CalendarDay>,
}
#[derive(Debug, Clone)]
pub struct QuoteRequest {
    pub symbols: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteResponse {
    pub request_id: Uuid,
    pub provider: String,
    pub quotes: Vec<serde_json::Value>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderHealth {
    pub available: bool,
}
