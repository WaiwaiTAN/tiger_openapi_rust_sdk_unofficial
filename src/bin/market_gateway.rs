use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    middleware::{self, Next},
    response::Response,
};
use clap::Parser;
use secrecy::{ExposeSecret, SecretString};
use std::{path::PathBuf, sync::Arc};
use tiger_market_data::gateway::{
    cache::MarketCache, config::GatewayConfig, realtime::RealtimeHub, router, state::AppState,
    tiger_provider::TigerMarketDataProvider,
};
use tigeropen::{config::ClientConfig, quote::QuoteClient};
#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "config/gateway.toml")]
    config: PathBuf,
    #[arg(long)]
    allow_remote: bool,
    #[arg(long)]
    api_token_source: Option<PathBuf>,
    #[arg(long)]
    json_logs: bool,
}
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    let mut diff = a.len() ^ b.len();
    let n = a.len().max(b.len());
    for i in 0..n {
        diff |= usize::from(*a.get(i).unwrap_or(&0) ^ *b.get(i).unwrap_or(&0));
    }
    diff == 0
}
async fn authorize(
    token: SecretString,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let supplied = request
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .unwrap_or("");
    if !constant_time_eq(supplied.as_bytes(), token.expose_secret().as_bytes()) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(next.run(request).await)
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "info,tower_http=info".into());
    if args.json_logs {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .json()
            .init()
    } else {
        tracing_subscriber::fmt().with_env_filter(filter).init()
    }
    let cfg = GatewayConfig::load(&args.config)?;
    cfg.validate_bind(args.allow_remote, args.api_token_source.as_deref())?;
    let mut builder = ClientConfig::builder();
    if let Some(path) = cfg
        .tiger
        .as_ref()
        .and_then(|tiger| tiger.credential_file.as_ref())
    {
        builder = builder.properties_file(&path.to_string_lossy());
    }
    let client_cfg = builder.build()?;
    if cfg.acquire_quote_permission_on_startup {
        QuoteClient::from_config(client_cfg.clone())
            .grab_quote_permission()
            .await?;
    }
    let realtime = match RealtimeHub::connect(client_cfg.clone()).await {
        Ok(hub) => Some(hub),
        Err(error) => {
            tracing::warn!(%error, "real-time push unavailable; REST gateway will continue");
            None
        }
    };
    let provider = Arc::new(TigerMarketDataProvider::new(client_cfg, 4));
    if let Some(parent) = cfg.cache_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let cache = Arc::new(MarketCache::open(&cfg.cache_path)?);
    let app = router(AppState {
        provider,
        cache,
        config: cfg.clone(),
        realtime,
    });
    let app = if let Some(path) = args.api_token_source {
        let value = std::fs::read_to_string(path)?.trim().to_owned();
        if value.is_empty() {
            anyhow::bail!("API token file is empty");
        }
        app.layer(middleware::from_fn(move |request, next| {
            authorize(SecretString::from(value.clone()), request, next)
        }))
    } else {
        app
    };
    let listener = tokio::net::TcpListener::bind(cfg.bind).await?;
    tracing::info!(bind=%cfg.bind,"market-data gateway listening");
    axum::serve(listener, app).await?;
    Ok(())
}
