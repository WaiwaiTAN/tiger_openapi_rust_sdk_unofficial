use serde_json::json;
use std::path::PathBuf;
use tiger_openapi_rust_sdk_unofficial::{
    client_config::ClientConfig, quote_client::QuoteClient, tiger_enums,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up client config with sandbox properties
    let mut cfg = ClientConfig::new();
    cfg.props_path = Some(PathBuf::from(
        std::env::var_os("TIGER_CREDENTIAL_DIRECTORY")
            .ok_or("TIGER_CREDENTIAL_DIRECTORY must explicitly name the credential directory")?,
    ));
    cfg.load_props()?;
    cfg.load_token()?;

    // Create QuoteClient
    let mut quote_client = QuoteClient::new(cfg, true).await?;

    match quote_client
        .get_history_timeline(
            &json!(vec!["000001.SH".to_string(), ".SPX".to_string()]),
            "2006-03-26",
            &tiger_enums::QuoteRight::br,
        )
        .await
    {
        Ok(result) => {
            println!(
                "Raw JSON result: {}",
                serde_json::to_string_pretty(&result)?
            );
        }
        Err(e) => {
            eprintln!("Error calling get_kline: {}", e);
        }
    }

    Ok(())
}
