use crate::client_config::ClientConfig;
use crate::constants;
use crate::models::{Kline, KlineItem};
use crate::service_types;
use crate::tiger_client::TigerClient;
use crate::tiger_enums;
use anyhow::Result;
use serde_json::{Map, Value};
use std::error::Error;

#[derive(Debug, Clone)]
pub struct QuoteClient {
    client: TigerClient,
}

impl QuoteClient {
    pub async fn new(cf: ClientConfig) -> Self {
        let client = TigerClient::new(cf);
        let qc = QuoteClient { client };
        qc.grab_quote_permission()
            .await
            .expect("Failed to grab quote permission");
        qc
    }

    pub async fn grab_quote_permission(&self) -> Result<Value> {
        let obj = Value::Object(serde_json::Map::new());
        self.client
            .post(service_types::GRAB_QUOTE_PERMISSION, obj)
            .await
    }

    pub async fn get_quote_permission(&self) -> Result<Value> {
        let obj = Value::Object(serde_json::Map::new());
        self.client
            .post(service_types::GET_QUOTE_PERMISSION, obj)
            .await
    }

    pub async fn get_symbols(
        &self,
        market: tiger_enums::Market,
        include_otc: bool,
    ) -> Result<Value> {
        let obj = serde_json::json!({
            constants::P_MARKET: market.to_str(),
            constants::P_INCLUDE_OTC: include_otc,
        });
        self.client.post(service_types::ALL_SYMBOLS, obj).await
    }

    pub async fn get_all_symbol_names(
        &self,
        market: tiger_enums::Market,
        include_otc: bool,
    ) -> Result<Value> {
        let obj = serde_json::json!({
            constants::P_MARKET: market.to_str(),
            constants::P_INCLUDE_OTC: include_otc,
        });
        self.client.post(service_types::ALL_SYMBOL_NAMES, obj).await
    }

    pub async fn get_market_state(&self, market: tiger_enums::Market) -> Result<Value> {
        let obj = serde_json::json!({
            constants::P_MARKET: market.to_str(),
        });
        self.client.post(service_types::MARKET_STATE, obj).await
    }

    pub async fn get_kline_quota(&self, with_details: bool) -> Result<Value> {
        let obj = serde_json::json!({
            constants::P_WITH_DETAILS: with_details,
        });
        self.client.post(service_types::KLINE_QUOTA, obj).await
    }

    pub async fn get_history_timeline(
        &mut self,
        symbols: &Value,
        date: &str,
        right: &tiger_enums::QuoteRight,
    ) -> Result<Value> {
        let obj = serde_json::json!({
            constants::P_SYMBOLS: symbols,
            constants::P_DATE: date,
            constants::P_RIGHT: right.to_str(),
        });
        self.client.post(service_types::HISTORY_TIMELINE, obj).await
    }

    pub async fn get_trading_calendar(
        &self,
        market: tiger_enums::Market,
        begin_date: &str,
        end_date: &str,
    ) -> Result<Value> {
        let obj = serde_json::json!({
            constants::P_MARKET: market.to_str(),
            constants::P_BEGIN_DATE: begin_date,
            constants::P_END_DATE: end_date,
        });
        self.client.post(service_types::TRADING_CALENDAR, obj).await
    }

    async fn get_kline_raw(
        &self,
        symbols: &Value,
        period: &str,
        begin_time: i64,
        end_time: i64,
        right: &str,
        limit: i32,
        page_token: &str,
    ) -> Result<Value> {
        let obj = serde_json::json!({
            constants::P_SYMBOLS: symbols,
            constants::P_PERIOD: period,
            constants::P_BEGIN_TIME: begin_time,
            constants::P_END_TIME: end_time,
            constants::P_RIGHT: right,
            constants::P_LIMIT: limit,
            constants::P_LANG: "en_US",
            constants::P_PAGE_TOKEN: page_token,
        });
        Ok(self.client.post(service_types::KLINE, obj).await?)
    }

    pub async fn get_kline(
        &self,
        symbols: &Value,
        period: Option<&str>,
        begin_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<i32>,
        right: Option<&str>,
        page_token: Option<&str>,
    ) -> Result<Vec<Kline>> {
        let period = period.unwrap_or("day");
        let begin_time = begin_time.unwrap_or(-1);
        let end_time = end_time.unwrap_or(-1);
        let limit = limit.unwrap_or(251);
        let right = right.unwrap_or("br");
        let page_token = page_token.unwrap_or("");
        let result = self
            .get_kline_raw(
                symbols, period, begin_time, end_time, right, limit, page_token,
            )
            .await?;
        let obj = result
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Expected array"))?;
        let mut klines = Vec::new();
        for bar in obj {
            let items = bar
                .get("items")
                .and_then(|v| v.as_array())
                .ok_or_else(|| anyhow::anyhow!("Expected items array"))?;
            let mut kline_items = Vec::new();
            for item in items {
                let kline_item: KlineItem = serde_json::from_value(item.clone())?;
                kline_items.push(kline_item);
            }
            let symbol = bar
                .get("symbol")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Expected symbol"))?
                .to_string();
            let period_str = bar
                .get("period")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Expected period"))?
                .to_string();
            let kline = Kline {
                symbol,
                period: period_str,
                items: kline_items,
                contract_code: None,
                expiry: None,
                right: None,
                strike: None,
            };
            klines.push(kline);
        }
        Ok(klines)
    }
}
