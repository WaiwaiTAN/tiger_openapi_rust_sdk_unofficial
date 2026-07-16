use super::{cache::MarketCache, config::GatewayConfig, provider::MarketDataProvider};
use std::sync::Arc;
#[derive(Clone)]
pub struct AppState {
    pub provider: Arc<dyn MarketDataProvider>,
    pub cache: Arc<MarketCache>,
    pub config: GatewayConfig,
}
