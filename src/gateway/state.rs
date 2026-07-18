use super::{
    cache::MarketCache, config::GatewayConfig, provider::MarketDataProvider, realtime::RealtimeHub,
};
use std::sync::Arc;
#[derive(Clone)]
pub struct AppState {
    pub provider: Arc<dyn MarketDataProvider>,
    pub cache: Arc<MarketCache>,
    pub config: GatewayConfig,
    pub realtime: Option<RealtimeHub>,
}
