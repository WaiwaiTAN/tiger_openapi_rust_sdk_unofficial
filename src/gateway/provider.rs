use super::{error::ProviderError, models::*};
use async_trait::async_trait;
#[async_trait]
pub trait MarketDataProvider: Send + Sync {
    fn provider_name(&self) -> &'static str;
    async fn health(&self) -> Result<ProviderHealth, ProviderError>;
    async fn bars(&self, request: BarsRequest) -> Result<BarsResponse, ProviderError>;
    async fn trading_calendar(
        &self,
        request: CalendarRequest,
    ) -> Result<CalendarResponse, ProviderError>;
    async fn quote(&self, _request: QuoteRequest) -> Result<QuoteResponse, ProviderError> {
        Err(ProviderError::Unsupported)
    }
}
