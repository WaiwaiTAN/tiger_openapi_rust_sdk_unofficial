use crate::client_config::ClientConfig;
use crate::constants;
use crate::service_types;
use crate::tiger_client::TigerClient;
use openssl::string;
use serde_json::{Map, Value, json};
use std::error::Error;
// use std::time::SystemTime;
use crate::models::Order;
use libc::time_t;
use anyhow::Result;
#[derive(Debug, Clone)]
pub struct TradeClient {
    client: TigerClient,
}

impl TradeClient {
    pub fn new(cf: ClientConfig) -> Self {
        let client = TigerClient::new(cf);
        let tc = TradeClient { client };
        tc
    }

    pub async fn prime_assets(&mut self) -> Result<Value> {
        let obj = json!({
            "account": self.client.client_config.account.clone(),
            "base_currency": "HKD",
            "consolidated": true,
            "lang": "en_US",
        });

        self.client.post(service_types::PRIME_ASSETS, obj).await
    }
    pub async fn place_order(
        &self,
        order: &mut Order,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let account_param = self.get_account_param(&order.account);
        let mut obj = order.to_value(
            account_param,
            if let Some(secret_key) = &order.secret_key {
                Some(secret_key.clone())
            } else {
                None
            },
        );

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
            Value::String(self.client.client_config.account.clone())
        } else {
            Value::String(account.to_string())
        }
    }

    fn set_secret_key(&self, obj: &mut Map<String, Value>) {
        if !self.client.client_config.secret_key.is_empty() {
            obj.insert(
                constants::P_SECRET_KEY.to_string(),
                Value::String(self.client.client_config.secret_key.clone()),
            );
        }
    }
}
