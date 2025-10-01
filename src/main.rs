// src/main.rs

mod constants;
mod properties;
mod client_config;
mod crypto_utils;
mod service_types;

use std::path::PathBuf;
use client_config::ClientConfig;

mod quote_client;
use quote_client::QuoteClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cfg = ClientConfig::new();
    cfg.props_path = Some( PathBuf::from("properties/"));

    // 从属性文件读入配置与环境
    cfg.load_props();

    // println!("{:?}", cfg);

    // 读入 token
    cfg.load_token();
    // 从 ClientConfig 转换成 QuoteClient
    let mut quote_client = QuoteClient::new(cfg, false);
    quote_client.grab_quote_permission().await?;

    Ok(())
}

