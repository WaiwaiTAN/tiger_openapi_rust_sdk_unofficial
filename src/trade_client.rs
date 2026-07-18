use crate::client_config::ClientConfig;
use crate::constants;
use crate::service_types;
use crate::tiger_client::TigerClient;
use serde_json::{Map, Value, json};
// use std::time::SystemTime;
use crate::models::Order;
use crate::tiger_enums::OrderSortBy;
use anyhow::Result;
use secrecy::ExposeSecret;

/// Filters accepted by Tiger's historical-orders endpoint.
///
/// Times are Unix timestamps in milliseconds. `start_time` is inclusive and
/// `end_time` is exclusive.
#[derive(Debug, Clone)]
pub struct HistoryOrdersRequest {
    pub account: Option<String>,
    pub sec_type: Option<String>,
    pub market: String,
    pub symbol: Option<String>,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub limit: u32,
    pub is_brief: bool,
    pub states: Option<Vec<String>>,
    pub sort_by: Option<OrderSortBy>,
    pub seg_type: Option<String>,
    pub page_token: Option<String>,
}

impl Default for HistoryOrdersRequest {
    fn default() -> Self {
        Self {
            account: None,
            sec_type: None,
            market: "ALL".to_string(),
            symbol: None,
            start_time: None,
            end_time: None,
            limit: 100,
            is_brief: false,
            states: None,
            sort_by: None,
            seg_type: None,
            page_token: None,
        }
    }
}

impl HistoryOrdersRequest {
    pub fn new() -> Self {
        Self::default()
    }

    fn to_value(&self, default_account: &str) -> Result<Value> {
        if self.limit == 0 {
            anyhow::bail!("history orders limit must be greater than zero");
        }
        if matches!((self.start_time, self.end_time), (Some(start), Some(end)) if start >= end) {
            anyhow::bail!("history orders start_time must be earlier than end_time");
        }

        let mut obj = Map::new();
        obj.insert(
            constants::P_ACCOUNT.to_string(),
            Value::String(
                self.account
                    .as_deref()
                    .filter(|account| !account.is_empty())
                    .unwrap_or(default_account)
                    .to_string(),
            ),
        );
        obj.insert(
            constants::P_MARKET.to_string(),
            Value::String(self.market.clone()),
        );
        obj.insert(constants::P_LIMIT.to_string(), json!(self.limit));
        obj.insert(constants::P_IS_BRIEF.to_string(), json!(self.is_brief));

        insert_optional_string(&mut obj, constants::P_SEC_TYPE, self.sec_type.as_deref());
        insert_optional_string(&mut obj, constants::P_SYMBOL, self.symbol.as_deref());
        insert_optional_string(&mut obj, constants::P_SEG_TYPE, self.seg_type.as_deref());
        if let Some(start_time) = self.start_time {
            obj.insert(constants::P_START_DATE.to_string(), json!(start_time));
        }
        if let Some(end_time) = self.end_time {
            obj.insert(constants::P_END_DATE.to_string(), json!(end_time));
        }
        if let Some(states) = &self.states {
            obj.insert(constants::P_STATES.to_string(), json!(states));
        }
        if let Some(sort_by) = self.sort_by {
            obj.insert(
                constants::P_SORT_BY.to_string(),
                Value::String(sort_by.to_str().to_string()),
            );
        }
        if let Some(page_token) = &self.page_token {
            obj.insert(
                constants::P_PAGE_TOKEN.to_string(),
                Value::String(page_token.clone()),
            );
        }
        Ok(Value::Object(obj))
    }
}

fn insert_optional_string(obj: &mut Map<String, Value>, key: &str, value: Option<&str>) {
    if let Some(value) = value.filter(|value| !value.is_empty()) {
        obj.insert(key.to_string(), Value::String(value.to_string()));
    }
}

#[derive(Debug, Clone)]
pub struct TradeClient {
    client: TigerClient,
}

impl TradeClient {
    pub fn new(cf: ClientConfig) -> Result<Self> {
        let client = TigerClient::new(cf)?;
        Ok(TradeClient { client })
    }

    pub async fn prime_assets(&mut self) -> Result<Value> {
        let obj = json!({
            "account": self.client.client_config.account.expose_secret(),
            "base_currency": "HKD",
            "consolidated": true,
            "lang": "en_US",
        });

        self.client.post(service_types::PRIME_ASSETS, obj).await
    }

    /// Fetches historical orders, including open, filled, cancelled, expired,
    /// and rejected orders, subject to the supplied filters.
    pub async fn get_history_orders(&self, request: &HistoryOrdersRequest) -> Result<Value> {
        let mut params = request.to_value(self.client.client_config.account.expose_secret())?;
        if let Some(obj) = params.as_object_mut() {
            self.set_secret_key(obj);
        }
        self.client.post(service_types::ORDERS, params).await
    }

    /// Alias matching Tiger's other SDKs.
    pub async fn get_orders(&self, request: &HistoryOrdersRequest) -> Result<Value> {
        self.get_history_orders(request).await
    }

    pub async fn place_order(
        &self,
        order: &mut Order,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let account_param = self.get_account_param(&order.account);
        let mut obj = order.to_value(account_param, order.secret_key.clone());

        self.set_secret_key(&mut obj);

        let res = self
            .client
            .post(service_types::PLACE_ORDER, Value::Object(obj))
            .await?;
        if let Some(id_val) = res.get("id").and_then(|v| v.as_u64()) {
            order.id = Some(id_val);
        } else {
            println!("Warn: id not returned");
        }
        if let Some(sub_ids) = res.get("subIds") {
            order.sub_ids = Some(sub_ids.clone());
        }
        Ok(res)
    }

    fn get_account_param(&self, account: &str) -> Value {
        if account.is_empty() {
            Value::String(
                self.client
                    .client_config
                    .account
                    .expose_secret()
                    .to_string(),
            )
        } else {
            Value::String(account.to_string())
        }
    }

    fn set_secret_key(&self, obj: &mut Map<String, Value>) {
        if !self
            .client
            .client_config
            .secret_key
            .expose_secret()
            .is_empty()
        {
            obj.insert(
                constants::P_SECRET_KEY.to_string(),
                Value::String(
                    self.client
                        .client_config
                        .secret_key
                        .expose_secret()
                        .to_string(),
                ),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_order_params_use_api_field_names_and_defaults() {
        let request = HistoryOrdersRequest {
            sec_type: Some("STK".to_string()),
            symbol: Some("AAPL".to_string()),
            start_time: Some(1_700_000_000_000),
            end_time: Some(1_700_086_400_000),
            states: Some(vec!["FILLED".to_string(), "CANCELLED".to_string()]),
            sort_by: Some(OrderSortBy::LATEST_STATUS_UPDATED),
            seg_type: Some("SEC".to_string()),
            page_token: Some("next-page".to_string()),
            ..HistoryOrdersRequest::default()
        };

        let value = request.to_value("default-account").unwrap();
        assert_eq!(value["account"], "default-account");
        assert_eq!(value["market"], "ALL");
        assert_eq!(value["sec_type"], "STK");
        assert_eq!(value["symbol"], "AAPL");
        assert_eq!(value["start_date"], 1_700_000_000_000_i64);
        assert_eq!(value["end_date"], 1_700_086_400_000_i64);
        assert_eq!(value["limit"], 100);
        assert_eq!(value["is_brief"], false);
        assert_eq!(value["states"], json!(["FILLED", "CANCELLED"]));
        assert_eq!(value["sort_by"], "LATEST_STATUS_UPDATED");
        assert_eq!(value["seg_type"], "SEC");
        assert_eq!(value["page_token"], "next-page");
    }

    #[test]
    fn history_order_params_validate_range_and_limit() {
        let request = HistoryOrdersRequest {
            limit: 0,
            ..HistoryOrdersRequest::default()
        };
        assert!(request.to_value("account").is_err());

        let request = HistoryOrdersRequest {
            start_time: Some(2),
            end_time: Some(1),
            ..HistoryOrdersRequest::default()
        };
        assert!(request.to_value("account").is_err());
    }
}
