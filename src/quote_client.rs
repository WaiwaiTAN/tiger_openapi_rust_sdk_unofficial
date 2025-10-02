// src/quote_client.rs
use crate::client_config::{self, ClientConfig};
use crate::constants;
use crate::crypto_utils;
use crate::service_types;

use chrono::Utc;

/// 行情客户端，基于 ClientConfig
#[derive(Debug, Clone)]
pub struct QuoteClient {
    pub config: ClientConfig,
    pub has_permission: bool,
}

use openssl::string;
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue, USER_AGENT};
use serde::{Deserialize, Serialize};

use serde_json::Map;
use std::collections::HashMap;

/// 通用请求包装，带有业务数据和签名信息
#[derive(Debug, Clone, Serialize)]
pub struct RequestWrapper<'a, T> {
    pub method: String,
    pub version: String,
    #[serde(skip)]
    pub biz_content_obj: Option<T>, // 业务内容（泛型）
    pub biz_content: String,
    pub timestamp: String, // 时间戳
    #[serde(flatten)]
    pub config: &'a ClientConfig, // 固定配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sign: Option<String>, // 可选，None 时不会出现在 JSON 中
}

/// 示例：某个业务的内容
#[derive(Debug, Serialize)]
struct AssetBiz {
    account: String,
    base_currency: String,
    consolidated: bool,
    lang: String,
}

#[derive(Debug, Deserialize)]
struct ApiResponse<T> {
    code: i32,
    message: Option<String>,
    data: Option<T>,
}

use base64::{Engine as _, engine::general_purpose};
use rsa::{RsaPublicKey, pkcs1v15::VerifyingKey, pkcs8::DecodePublicKey, signature::Verifier};
use serde_json::Value;
use sha2::Sha256;

#[derive(Debug)]
pub struct ResponseException(pub String);

/// 等价于 Python 的 TigerResponse
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TigerResponse {
    pub code: Option<i32>,
    pub message: Option<String>,
    pub data: Option<Value>, // 用 serde_json::Value 来存储动态 JSON
}

impl TigerResponse {
    pub fn is_success(&self) -> bool {
        self.code == Some(0)
    }

    pub fn parse_response_content(&mut self, response: &Value) -> Value {
        if let Some(code) = response.get("code").and_then(|v| v.as_i64()) {
            self.code = Some(code as i32);
        }
        if let Some(msg) = response.get("message").and_then(|v| v.as_str()) {
            self.message = Some(msg.to_string());
        }
        if let Some(data) = response.get("data") {
            if data.is_string() {
                if let Ok(parsed) = serde_json::from_str::<Value>(data.as_str().unwrap()) {
                    self.data = Some(parsed);
                }
            } else {
                self.data = Some(data.clone());
            }
        }
        response.clone()
    }
}

/// 等价于 Python 的 FundDetailsResponse
#[derive(Debug, Default)]
pub struct FundDetailsResponse {
    pub base: TigerResponse,
    pub result: Vec<HashMap<String, Value>>, // 用 Vec<HashMap> 代替 DataFrame
    pub is_success: Option<bool>,
}

impl FundDetailsResponse {
    pub fn new() -> Self {
        FundDetailsResponse {
            base: TigerResponse::default(),
            result: Vec::new(),
            is_success: None,
        }
    }

    pub fn parse_response_content(&mut self, response_content: &Value) {
        let response = self.base.parse_response_content(response_content);

        if let Some(success) = response.get("is_success").and_then(|v| v.as_bool()) {
            self.is_success = Some(success);
        }

        if let Some(data) = &self.base.data {
            if let Some(obj) = data.as_object() {
                // 模拟 camel_to_underline 转换
                let mut normalized: HashMap<String, Value> = HashMap::new();
                for (k, v) in obj {
                    normalized.insert(camel_to_underline(k), v.clone());
                }

                if let Some(items) = normalized.get("items").and_then(|v| v.as_array()) {
                    self.result = items
                        .iter()
                        .filter_map(|item| item.as_object())
                        .map(|map| {
                            map.iter()
                                .map(|(k, v)| (camel_to_underline(k), v.clone()))
                                .collect::<HashMap<String, Value>>()
                        })
                        .collect();

                    // 给每一行加上 page/limit/item_count/page_count/timestamp
                    for row in &mut self.result {
                        for key in ["page", "limit", "item_count", "page_count", "timestamp"] {
                            if let Some(val) = normalized.get(key) {
                                row.insert(key.to_string(), val.clone());
                            }
                        }
                    }
                }
            }
        }
    }
}

/// 简单实现 camelCase -> snake_case
fn camel_to_underline(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

#[derive(Debug, Deserialize)]
struct GrabQuotePermissionData {}

use std::collections::BTreeMap;

/// 将 RequestWrapper 序列化为 URL 查询参数格式
pub fn serialize_request<T: Serialize>(wrapper: &RequestWrapper<T>) -> String {
    let mut params = BTreeMap::new();

    // 添加 config 中的字段（除 skip 的字段）
    params.insert("tiger_id", wrapper.config.tiger_id.clone());
    params.insert("charset", wrapper.config.charset.clone());
    params.insert("version", wrapper.version.clone());
    params.insert("sign_type", wrapper.config.sign_type.clone());
    params.insert("device_id", wrapper.config.device_id.clone());

    // 添加 wrapper 中的字段
    params.insert("method", wrapper.method.clone());
    params.insert("timestamp", wrapper.timestamp.clone());

    if let Some(sign) = &wrapper.sign {
        params.insert("sign", sign.clone());
    }

    if let Some(biz_content) = &wrapper.biz_content_obj {
        let json = serde_json::to_string(&biz_content).unwrap();
        params.insert("biz_content", json);
    }

    // 拼接为查询字符串格式
    params
        .into_iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&")
}

impl QuoteClient {
    /// 使用 ClientConfig 初始化 QuoteClient
    pub fn new(config: ClientConfig, grab_permission: bool) -> Self {
        let mut client = Self {
            config,
            has_permission: false,
        };

        if grab_permission {}

        client
    }

    pub fn parse_response(
        &mut self,
        response_str: &str,
        ts: &str,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        // 解析 JSON
        let response_content: Value = serde_json::from_str(response_str)?;

        // 如果没有公钥、没有 sign 或没有 timestamp，则直接返回
        if self.config.server_public_key.is_empty() || !response_content.get("sign").is_some() {
            return Ok(response_content);
        }

        let sign = response_content["sign"].as_str().unwrap_or("");

        // 验签
        let verify_res =
            crypto_utils::sha1_verify(ts, sign, &self.config.server_public_key.as_str())?;
        if !verify_res {
            return Err(format!("response sign verify failed: {}", response_str).into());
        }

        Ok(response_content)
    }

    /// 发起真实的 HTTP POST 请求以获取行情权限
    pub async fn grab_quote_permission(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let url = &self.config.server_url;

        // 构建 headers
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/json; charset=UTF-8"),
        );
        headers.insert(
            USER_AGENT,
            HeaderValue::from_str(&format!(
                "{}{}",
                constants::P_SDK_VERSION_PREFIX,
                constants::PROJECT_VERSION
            ))?,
        );

        if !self.config.token.is_empty() {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(self.config.token.as_str())?,
            );
        }

        // 构建 client
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        let biz = AssetBiz {
            account: "65568723".into(),
            base_currency: "HKD".into(),
            consolidated: true,
            lang: "en_US".into(),
        };

        let mut body = RequestWrapper {
            config: &self.config,
            method: service_types::PRIME_ASSETS.to_string(),
            version: constants::OPEN_API_SERVICE_VERSION.to_string(),
            biz_content_obj: Some(biz),
            biz_content: "".to_string(),
            timestamp: Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            sign: None,
        };

        body.biz_content = serde_json::to_string(&body.biz_content_obj).unwrap();

        // 2. 序列化（不含 sign）
        let json_without_sign = serialize_request(&body);
        println!("content need to be signed:\n{}", json_without_sign);

        // 3. 计算签名
        let signature = crypto_utils::get_sign(&self.config.private_key, &json_without_sign)?;
        println!("calculated sign:\n{:?}", signature);

        // 4. 填入签名
        body.sign = Some(signature);

        println!("body:\n{}", serde_json::to_string(&body)?);
        let resp = client
            .post(url)
            .body(serde_json::to_string(&body)?)
            .send()
            .await?;

        // 解析返回
        let status = resp.status();
        if !status.is_success() {
            return Err(format!("HTTP error: {}", status).into());
        }
        let content = resp.text().await?;
        if let Ok(value) = self.parse_response(content.as_str(), body.timestamp.as_str()) {
            // 如果解析成功，继续往下执行
            println!("解析成功: {:?}", value);

            
        } else {
            println!("Failed to verify the message!");
        };

        let code = 0;
        // 约定：code == 0 表示成功；如不一致，请按服务端文档调整
        if code == 0 {
            self.has_permission = true;
            Ok(())
        } else {
            // let msg = api_resp
            //     .message
            //     .unwrap_or_else(|| "unknown error".to_string());
            // Err(format!("API error code {}: {}", api_resp.code, msg).into())
            Err(format!("API error code {}: {}", 0, "nothing").into())
        }
    }
}
