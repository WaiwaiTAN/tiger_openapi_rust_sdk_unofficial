use crate::client_config::ClientConfig;
use crate::constants;
use crate::contract_utils;
use crate::tiger_utils;

use anyhow::{Result, anyhow};
use reqwest::{
    Client, Method, Response,
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue, USER_AGENT},
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::error::Error;
#[derive(Debug, Clone)]
pub struct TigerClient {
    pub client_config: ClientConfig,
}

impl TigerClient {
    pub fn new(cf: ClientConfig) -> Self {
        TigerClient { client_config: cf }
    }

    pub fn set_config(&mut self, cf: ClientConfig) {
        self.client_config = cf;
        self.client_config.check();
    }

    fn build_sign_content(&self, obj: &Value) -> String {
        // 确保 obj 是一个对象
        let obj_obj = obj.as_object().expect("Expected JSON object");
        // 收集所有 key
        let mut keys: Vec<&String> = obj_obj.keys().collect();
        // 排序
        keys.sort();
        // 拼接字符串
        let mut result = String::new();
        for key in keys {
            if !result.is_empty() {
                result.push('&');
            }
            let val = obj_obj.get(key).unwrap();
            // 假设 value 是字符串类型
            let val_str = val.as_str().expect("Expected string value");
            result.push_str(key);
            result.push('=');
            result.push_str(val_str);
        }
        result
    }

    fn build_common_params(&self) -> Value {
        serde_json::json!({
            constants::P_TIGER_ID: self.client_config.tiger_id,
            constants::P_CHARSET:self.client_config.charset,
            constants::P_VERSION: constants::OPEN_API_SERVICE_VERSION,
            constants::P_SIGN_TYPE: self.client_config.sign_type,
            constants::P_DEVICE_ID: self.client_config.device_id,
        })
    }

    pub async fn post(&self, api_method: &str, params: Value) -> Result<Value> {
        self.send_request(Method::POST, api_method, params).await
    }

    pub async fn get(&self, api_method: &str, params: Value) -> Result<Value> {
        self.send_request(Method::GET, api_method, params).await
    }

    pub async fn send_request(
        &self,
        http_method: Method,
        api_method: &str,
        mut body: Value,
    ) -> Result<Value> {
        // 构造请求头
        let client = Client::new();
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(ACCEPT, "application/json".parse()?);
        headers.insert(CONTENT_TYPE, "application/json; charset=UTF-8".parse()?);
        headers.insert(
            USER_AGENT,
            HeaderValue::from_str(&format!(
                "{}{}",
                constants::P_SDK_VERSION_PREFIX,
                constants::PROJECT_VERSION
            ))?,
        );
        if !self.client_config.token.is_empty() {
            headers.insert(AUTHORIZATION, self.client_config.token.parse()?);
        }

        // 构造请求体
        let mut params = json!({});
        let common_params = self.build_common_params();
        if let Some(obj) = common_params.as_object() {
            for (k, v) in obj {
                params[k] = v.clone();
            }
        }
        if !self.client_config.lang.is_empty() {
            body["lang"] = Value::String(self.client_config.lang.clone());
        }
        if !body.is_null() && body.as_object().map_or(0, |o| o.len()) > 0 {
            params["biz_content"] = Value::String(body.to_string());
        }
        params["method"] = Value::String(api_method.to_string());
        params["timestamp"] = Value::String(tiger_utils::get_timestamp());

        let sign_content = self.build_sign_content(&params);
        let sign = tiger_utils::get_sign(&self.client_config.private_key, &sign_content)?;
        params["sign"] = Value::String(sign);

        // 发送请求
        let response: Response = client
            .request(http_method, &self.client_config.server_url)
            .headers(headers)
            .json(&params)
            .send()
            .await?;

        // 解析响应
        let result: Value = response.json().await?;
        let result_str = result.to_string();

        if result["code"].is_null() {
            return Err(anyhow!(format!("api error, response: {}", result_str)));
        }
        let code = result["code"].as_i64().unwrap_or(-1);
        if code != 0 {
            return Err(anyhow!(format!("api code error, response: {}", result_str)));
        }

        let res_sign = result["sign"].as_str().unwrap_or("");

        match tiger_utils::verify_sign(
            &self.client_config.server_public_key,
            params["timestamp"].as_str().unwrap_or(""),
            res_sign,
        ) {
            Ok(valid) => {
                if !valid {
                    println!("签名验证失败 ❌\nResponse: {}", result_str);
                }
            }
            Err(e) => {
                println!("验证过程中出错: {}", e);
            }
        }

        Ok(result["data"].clone())
    }

    pub fn identifiers_to_options(&self, identifiers: Value) -> Value {
        let mut options: Vec<Value> = Vec::new();
        if let Some(arr) = identifiers.as_array() {
            for identifier in arr {
                let ident_str = identifier.as_str().unwrap_or("");
                let (symbol, expiry, right, strike) =
                    contract_utils::ContractUtil::extract_option_info(ident_str);
                if symbol.is_empty() || expiry.is_empty() || right.is_empty() {
                    continue;
                }

                let obj = json!({ "expiry": tiger_utils::date_string_to_timestamp(&expiry), "right": right, "strike": strike, "symbol": symbol, });
                options.push(obj);
            }
        }
        Value::Array(options)
    }
}
