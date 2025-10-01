use std::time::{SystemTime, UNIX_EPOCH, Duration};
use std::collections::HashMap;

use reqwest::{Client, StatusCode};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

use crate::constants;

/// 错误类型（可根据你的 C++ 行为扩展）
#[derive(Debug)]
pub enum TigerError {
    Http(StatusCode, String),
    Reqwest(reqwest::Error),
    Json(serde_json::Error),
    Sign(String),
    Other(String),
}

impl From<reqwest::Error> for TigerError {
    fn from(err: reqwest::Error) -> Self {
        TigerError::Reqwest(err)
    }
}

impl From<serde_json::Error> for TigerError {
    fn from(err: serde_json::Error) -> Self {
        TigerError::Json(err)
    }
}

/// 返回通用响应包（示例）
#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub message: Option<String>,
    pub data: Option<T>,
}

/// 示例数据结构（根据你的 C++ 对应类型替换）
#[derive(Debug, Serialize, Deserialize)]
pub struct Quote {
    pub symbol: String,
    pub last_price: f64,
    pub timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderRequest {
    pub symbol: String,
    pub quantity: i64,
    pub side: String,   // "BUY"/"SELL"
    pub type_: String,  // "LIMIT"/"MARKET" 等
    pub price: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderResponse {
    pub order_id: String,
    pub status: String,
}

/// Tiger 客户端
#[derive(Clone)]
pub struct TigerClient {
    client: Client,
    base_url: String,
    app_key: String,
    secret_key: String,
    api_version: String,
    default_timeout: Duration,
}

impl TigerClient {
    /// 构造函数：按需设置连接池、超时、UA 等
    pub fn new(
        base_url: impl Into<String>,
        app_key: impl Into<String>,
        secret_key: impl Into<String>,
        api_version: impl Into<String>,
    ) -> Result<Self, TigerError> {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .tcp_keepalive(Duration::from_secs(30))
            .pool_idle_timeout(Duration::from_secs(90))
            .build()
            .map_err(TigerError::Reqwest)?;

        Ok(Self {
            client,
            base_url: base_url.into(),
            app_key: app_key.into(),
            secret_key: secret_key.into(),
            api_version: api_version.into(),
            default_timeout: Duration::from_secs(10),
        })
    }

    /// 统一构造公共 headers（含签名/时间戳等）
    fn build_headers(
        &self,
        path: &str,
        method: &str,
        query: Option<&HashMap<String, String>>,
        body_json: Option<&serde_json::Value>,
        timestamp: u64,
    ) -> Result<HeaderMap, TigerError> {
        let mut headers = HeaderMap::new();

        headers.insert("X-APP-KEY", HeaderValue::from_str(&self.app_key).unwrap());
        headers.insert("X-TIMESTAMP", HeaderValue::from_str(&timestamp.to_string()).unwrap());
        headers.insert(CONTENT_TYPE, HeaderValue::from_static(CONTENT_TYPE_JSON));

        // 计算签名（占位）
        let sign = self.sign_request(path, method, query, body_json, timestamp)?;
        headers.insert("X-SIGNATURE", HeaderValue::from_str(&sign).unwrap());

        Ok(headers)
    }

    /// 签名函数（替换为你 C++ 的实际算法）
    fn sign_request(
        &self,
        path: &str,
        method: &str,
        query: Option<&HashMap<String, String>>,
        body_json: Option<&serde_json::Value>,
        timestamp: u64,
    ) -> Result<String, TigerError> {
        // 示例：将关键字段拼接后进行 HMAC-SHA256，再 hex/BASE64
        // 注意：这里是占位逻辑，请用真实的签名规则替换（字段顺序、分隔符、编码都需要与服务端一致）
        let mut canonical = String::new();
        canonical.push_str(method);
        canonical.push('\n');
        canonical.push_str(path);
        canonical.push('\n');

        if let Some(q) = query {
            // 需要与 C++ 保持同样的排序与编码
            let mut pairs: Vec<(&String, &String)> = q.iter().collect();
            pairs.sort_by(|a, b| a.0.cmp(b.0));
            for (k, v) in pairs {
                canonical.push_str(k);
                canonical.push('=');
                canonical.push_str(v);
                canonical.push('&');
            }
            if canonical.ends_with('&') {
                canonical.pop();
            }
        }
        canonical.push('\n');

        if let Some(b) = body_json {
            canonical.push_str(&b.to_string());
        }
        canonical.push('\n');

        canonical.push_str(&timestamp.to_string());

        // TODO: 用真实的 HMAC-SHA256
        // 例如：hmac_sha256(SECRET_KEY, canonical) -> hex/base64
        // 下面返回占位，避免误导
        Ok(format!("SIGN({})", canonical))
    }

    /// GET 示例：拉取行情
    pub async fn get_quotes(&self, symbols: &[&str]) -> Result<Vec<Quote>, TigerError> {
        let path = format!("/{}/quotes", self.api_version);
        let url = format!("{}{}", self.base_url, path);

        let mut query: HashMap<String, String> = HashMap::new();
        query.insert("symbols".to_string(), symbols.join(","));

        let timestamp = current_millis();

        let headers = self.build_headers(&path, "GET", Some(&query), None, timestamp)?;

        let resp = self.client
            .get(&url)
            .headers(headers)
            .query(&query)
            .timeout(self.default_timeout)
            .send()
            .await?;

        let status = resp.status();
        let text = resp.text().await?;

        if !status.is_success() {
            return Err(TigerError::Http(status, text));
        }

        let api: ApiResponse<Vec<Quote>> = serde_json::from_str(&text)?;
        if api.code != 0 {
            return Err(TigerError::Other(api.message.unwrap_or_else(|| "unknown error".into())));
        }
        Ok(api.data.unwrap_or_default())
    }

    /// POST 示例：下单
    pub async fn place_order(&self, req: &OrderRequest) -> Result<OrderResponse, TigerError> {
        let path = format!("/{}/orders/place", self.api_version);
        let url = format!("{}{}", self.base_url, path);

        let body = serde_json::to_value(req)?;
        let timestamp = current_millis();
        let headers = self.build_headers(&path, "POST", None, Some(&body), timestamp)?;

        let resp = self.client
            .post(&url)
            .headers(headers)
            .json(&req)
            .timeout(self.default_timeout)
            .send()
            .await?;

        let status = resp.status();
        let text = resp.text().await?;

        if !status.is_success() {
            return Err(TigerError::Http(status, text));
        }

        let api: ApiResponse<OrderResponse> = serde_json::from_str(&text)?;
        if api.code != 0 {
            return Err(TigerError::Other(api.message.unwrap_or_else(|| "unknown error".into())));
        }
        Ok(api.data.expect("order response missing"))
    }

    /// DELETE/POST 示例：撤单（按你的 C++ 实现替换）
    pub async fn cancel_order(&self, order_id: &str) -> Result<(), TigerError> {
        let path = format!("/{}/orders/cancel", self.api_version);
        let url = format!("{}{}", self.base_url, path);

        let mut payload = HashMap::new();
        payload.insert("order_id", order_id.to_string());

        let body = serde_json::to_value(&payload)?;
        let timestamp = current_millis();
        let headers = self.build_headers(&path, "POST", None, Some(&body), timestamp)?;

        let resp = self.client
            .post(&url)
            .headers(headers)
            .json(&payload)
            .timeout(self.default_timeout)
            .send()
            .await?;

        let status = resp.status();
        let text = resp.text().await?;

        if !status.is_success() {
            return Err(TigerError::Http(status, text));
        }

        let api: ApiResponse<serde_json::Value> = serde_json::from_str(&text)?;
        if api.code != 0 {
            return Err(TigerError::Other(api.message.unwrap_or_else(|| "unknown error".into())));
        }
        Ok(())
    }
}

fn current_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_millis() as u64
}
