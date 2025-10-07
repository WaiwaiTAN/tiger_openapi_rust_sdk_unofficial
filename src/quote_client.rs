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

use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue, USER_AGENT};
use serde::de::DeserializeOwned;
use serde::ser::Error;
use serde::{Deserialize, Serialize};

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

use std::collections::BTreeMap;
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub data: Option<T>,
    pub message: String,
    pub sign: String,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountData {
    pub account_id: String,
    pub segments: Vec<Segment>,
    pub update_timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Segment {
    pub buying_power: f64,
    pub capability: String,
    pub cash_available_for_trade: f64,
    pub cash_balance: f64,
    pub category: String,
    pub consolidated_seg_types: Vec<String>,
    pub currency: String,
    pub currency_assets: Vec<CurrencyAsset>,
    pub equity_with_loan: f64,
    pub excess_liquidation: f64,
    pub gross_position_value: f64,
    pub init_margin: f64,
    pub leverage: f64,
    pub locked_funds: f64,
    pub maintain_margin: f64,
    pub net_liquidation: f64,
    pub overnight_liquidation: f64,
    pub overnight_margin: f64,
    #[serde(rename = "realizedPL")]
    pub realized_pl: f64,
    #[serde(rename = "totalTodayPL")]
    pub total_today_pl: f64,
    #[serde(rename = "unrealizedPL")]
    pub unrealized_pl: f64,
    #[serde(rename = "unrealizedPLByCostOfCarry")]
    pub unrealized_pl_by_cost_of_carry: f64,
    pub uncollected: f64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyAsset {
    pub cash_available_for_trade: f64,
    pub cash_balance: f64,
    pub currency: String,
}

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
        let client = Self {
            config,
            has_permission: false,
        };

        if grab_permission {}

        client
    }

    pub fn parse_response<T>(
        &mut self,
        response_str: &str,
        ts: &str,
    ) -> Result<ApiResponse<T>, serde_json::Error>
    where
        T: DeserializeOwned,
    {
        // 解析 JSON
        if let Ok(response_content) = serde_json::from_str::<ApiResponse<T>>(response_str) {
            // 如果没有公钥、没有 sign 或没有 timestamp，则直接返回
            if self.config.server_public_key.is_empty() || !response_content.sign.is_empty() {
                return Ok(response_content);
            }

            let sign = &response_content.sign.as_str();

            // 验签
            match crypto_utils::sha1_verify(ts, sign, &self.config.server_public_key) {
                Ok(true) => return Ok(response_content),
                Ok(false) | Err(_) => {
                    return Err(serde_json::Error::custom("response sign verify failed"));
                }
            }
        }

        // JSON 解析失败
        Err(serde_json::Error::custom("response parse failed"))
    }

    /// 发起真实的 HTTP POST 请求以获取行情权限
    pub async fn prime_assets(&mut self) -> Result<f64, Box<dyn std::error::Error>> {
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

        // 3. 计算签名
        let signature = crypto_utils::get_sign(&self.config.private_key, &json_without_sign)?;

        // 4. 填入签名
        body.sign = Some(signature);

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
        let resp: ApiResponse<AccountData> =
            self.parse_response(content.as_str(), body.timestamp.as_str())?;

        if resp.code == 0 {
            if let Some(account_data) = resp.data {
                Ok(account_data.segments[0].buying_power)
            } else {
                Err(format!("Server Error, No account data recv").into())
            }
        } else {
            Err(format!("API error code {}: {}", resp.code, resp.message).into())
        }
    }
}
