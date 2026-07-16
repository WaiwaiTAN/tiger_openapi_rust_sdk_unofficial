use crate::{client_config::ClientConfig, constants, contract_utils, tiger_utils};
use anyhow::{Context, Result, anyhow};
use reqwest::{
    Client, Method,
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue, USER_AGENT},
};
use secrecy::ExposeSecret;
use serde_json::{Value, json};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct TigerClient {
    pub client_config: ClientConfig,
    http: Client,
}
impl TigerClient {
    pub fn new(cf: ClientConfig) -> Result<Self> {
        cf.validate_quote()?;
        let http = Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(15))
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(8)
            .redirect(reqwest::redirect::Policy::none())
            .user_agent(format!(
                "{}{}",
                constants::P_SDK_VERSION_PREFIX,
                constants::PROJECT_VERSION
            ))
            .build()?;
        Ok(Self {
            client_config: cf,
            http,
        })
    }
    pub fn set_config(&mut self, cf: ClientConfig) -> Result<()> {
        cf.validate_quote()?;
        self.client_config = cf;
        Ok(())
    }
    fn build_sign_content(obj: &Value) -> Result<String> {
        let map = obj
            .as_object()
            .ok_or_else(|| anyhow!("signing parameters must be an object"))?;
        let mut keys: Vec<_> = map.keys().collect();
        keys.sort();
        keys.into_iter()
            .map(|k| {
                map[k]
                    .as_str()
                    .map(|v| format!("{k}={v}"))
                    .ok_or_else(|| anyhow!("signing parameter {k} must be a string"))
            })
            .collect::<Result<Vec<_>>>()
            .map(|v| v.join("&"))
    }
    fn common_params(&self) -> Value {
        json!({constants::P_TIGER_ID:self.client_config.tiger_id.expose_secret(),constants::P_CHARSET:self.client_config.charset,constants::P_VERSION:constants::OPEN_API_SERVICE_VERSION,constants::P_SIGN_TYPE:self.client_config.sign_type,constants::P_DEVICE_ID:self.client_config.device_id})
    }
    pub async fn post(&self, method: &str, params: Value) -> Result<Value> {
        self.send_request(Method::POST, method, params).await
    }
    pub async fn get(&self, method: &str, params: Value) -> Result<Value> {
        self.send_request(Method::GET, method, params).await
    }
    pub async fn send_request(
        &self,
        http_method: Method,
        api_method: &str,
        mut body: Value,
    ) -> Result<Value> {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/json; charset=UTF-8"),
        );
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static("tiger-openapi-rust-unofficial/0.1"),
        );
        if !self.client_config.token.expose_secret().is_empty() {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(self.client_config.token.expose_secret())
                    .context("invalid authorization token encoding")?,
            );
        }
        let mut params = self.common_params();
        if !self.client_config.lang.is_empty() {
            body["lang"] = Value::String(self.client_config.lang.clone());
        }
        if body.as_object().is_some_and(|o| !o.is_empty()) {
            params["biz_content"] = Value::String(body.to_string());
        }
        params["method"] = Value::String(api_method.into());
        params["timestamp"] = Value::String(tiger_utils::get_timestamp());
        let sign_content = Self::build_sign_content(&params)?;
        params["sign"] = Value::String(tiger_utils::get_sign(
            self.client_config.private_key.expose_secret(),
            &sign_content,
        )?);
        let response = self
            .http
            .request(http_method, &self.client_config.server_url)
            .headers(headers)
            .json(&params)
            .send()
            .await?
            .error_for_status()?;
        let result: Value = response
            .json()
            .await
            .context("upstream returned malformed JSON")?;
        let code = result
            .get("code")
            .and_then(Value::as_i64)
            .ok_or_else(|| anyhow!("upstream response omitted result code"))?;
        if code != 0 {
            return Err(anyhow!("upstream API returned code {code}"));
        }
        Ok(result.get("data").cloned().unwrap_or(Value::Null))
    }
    pub fn identifiers_to_options(&self, identifiers: Value) -> Value {
        Value::Array(identifiers.as_array().into_iter().flatten().filter_map(|i|{let (symbol,expiry,right,strike)=contract_utils::ContractUtil::extract_option_info(i.as_str()?);if symbol.is_empty()||expiry.is_empty()||right.is_empty(){None}else{Some(json!({"expiry":tiger_utils::date_string_to_timestamp(&expiry),"right":right,"strike":strike,"symbol":symbol}))}}).collect())
    }
}
