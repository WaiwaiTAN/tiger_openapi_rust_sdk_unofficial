use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;
#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("provider is not configured")]
    NotConfigured,
    #[error("request is not supported")]
    Unsupported,
    #[error("upstream authentication failed")]
    Authentication,
    #[error("upstream rate limited the request")]
    RateLimited,
    #[error("upstream timed out")]
    Timeout,
    #[error("upstream request failed")]
    Upstream,
    #[error("upstream response was invalid")]
    Parse,
}
#[derive(Debug, Error)]
pub enum GatewayError {
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    Provider(#[from] ProviderError),
    #[error("cache unavailable")]
    Cache,
    #[error("not implemented")]
    NotImplemented,
}
#[derive(Serialize)]
struct ErrorEnvelope {
    error: ErrorBody,
}
#[derive(Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
    request_id: Uuid,
    retryable: bool,
}
impl IntoResponse for GatewayError {
    fn into_response(self) -> Response {
        let (status, code, retryable) = match &self {
            Self::Validation(_) => (StatusCode::BAD_REQUEST, "INVALID_REQUEST", false),
            Self::Provider(ProviderError::RateLimited) => {
                (StatusCode::BAD_GATEWAY, "UPSTREAM_RATE_LIMITED", true)
            }
            Self::Provider(ProviderError::Timeout) => {
                (StatusCode::GATEWAY_TIMEOUT, "UPSTREAM_TIMEOUT", true)
            }
            Self::Provider(ProviderError::Authentication) => (
                StatusCode::SERVICE_UNAVAILABLE,
                "PROVIDER_UNAVAILABLE",
                false,
            ),
            Self::Provider(ProviderError::Unsupported) | Self::NotImplemented => {
                (StatusCode::NOT_IMPLEMENTED, "NOT_IMPLEMENTED", false)
            }
            Self::Provider(_) => (StatusCode::BAD_GATEWAY, "UPSTREAM_ERROR", false),
            Self::Cache => (StatusCode::SERVICE_UNAVAILABLE, "CACHE_UNAVAILABLE", true),
        };
        (
            status,
            Json(ErrorEnvelope {
                error: ErrorBody {
                    code,
                    message: self.to_string(),
                    request_id: Uuid::new_v4(),
                    retryable,
                },
            }),
        )
            .into_response()
    }
}
