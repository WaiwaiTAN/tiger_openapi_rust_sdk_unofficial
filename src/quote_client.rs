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

/// 通用请求包装，带有业务数据和签名信息
#[derive(Debug, Clone, Serialize)]
pub struct RequestWrapper<'a, T> {
    pub method: String,
    pub version: String,
    #[serde(skip)]
    pub biz_content_obj: Option<T>, // 业务内容（泛型）
    pub biz_content: String,
    pub timestamp: String,      // 时间戳
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

/// 示例：另一个业务的内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBiz {
    pub user_id: String,
    pub nickname: String,
}

#[derive(Debug, Deserialize)]
struct ApiResponse<T> {
    code: i32,
    message: Option<String>,
    data: Option<T>,
}

#[derive(Debug, Deserialize)]
struct GrabQuotePermissionData {
    // 根据服务端实际返回结构补充字段
    // 例如：permission: bool
    // permission: Option<bool>,
}

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

        // let json_without_sign = "biz_content={\"account\":\"65568723\",\"base_currency\":\"HKD\",\"consolidated\":true,\"lang\":\"en_US\"}&charset=UTF-8&device_id=00:15:5d:c6:84:86&method=prime_assets&sign_type=RSA&tiger_id=20155322&timestamp=2025-10-01 22:49:49&version=2.0".to_string();

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

        let api_resp: ApiResponse<GrabQuotePermissionData> = resp.json().await?;
        // 约定：code == 0 表示成功；如不一致，请按服务端文档调整
        if api_resp.code == 0 {
            self.has_permission = true;
            println!("{:?}", api_resp.data);
            Ok(())
        } else {
            let msg = api_resp
                .message
                .unwrap_or_else(|| "unknown error".to_string());
            Err(format!("API error code {}: {}", api_resp.code, msg).into())
        }
    }
}
