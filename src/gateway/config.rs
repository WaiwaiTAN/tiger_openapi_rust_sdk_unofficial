use serde::Deserialize;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
    time::Duration,
};
use thiserror::Error;
fn default_bind() -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8765)
}
fn d_timeout() -> u64 {
    15
}
fn d_span() -> i64 {
    3650
}
fn d_limit() -> usize {
    10_000
}
#[derive(Debug, Clone, Deserialize)]
pub struct GatewayConfig {
    #[serde(default = "default_bind")]
    pub bind: SocketAddr,
    #[serde(default = "provider")]
    pub provider: String,
    pub cache_path: PathBuf,
    #[serde(default = "d_timeout")]
    pub request_timeout_seconds: u64,
    #[serde(default = "d_span")]
    pub max_date_span_days: i64,
    #[serde(default = "d_limit")]
    pub max_bars_per_response: usize,
    #[serde(default)]
    pub acquire_quote_permission_on_startup: bool,
    pub tiger: Option<TigerConfig>,
}
fn provider() -> String {
    "tiger".into()
}
#[derive(Debug, Clone, Deserialize)]
pub struct TigerConfig {
    /// Optional official Tiger properties file. TIGEROPEN_* environment variables
    /// take precedence and require no file.
    pub credential_file: Option<PathBuf>,
    #[serde(default)]
    pub environment: String,
}
#[derive(Debug, Error)]
pub enum GatewayConfigError {
    #[error("configuration read failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("configuration is invalid: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("remote binding requires --allow-remote and a secure API token source")]
    UnsafeBind,
}
impl GatewayConfig {
    pub fn load(path: &std::path::Path) -> Result<Self, GatewayConfigError> {
        Ok(toml::from_str(&std::fs::read_to_string(path)?)?)
    }
    pub fn validate_bind(
        &self,
        allow_remote: bool,
        token_source: Option<&std::path::Path>,
    ) -> Result<(), GatewayConfigError> {
        if !self.bind.ip().is_loopback() && (!allow_remote || token_source.is_none()) {
            Err(GatewayConfigError::UnsafeBind)
        } else {
            Ok(())
        }
    }
    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.request_timeout_seconds)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_is_loopback() {
        assert!(default_bind().ip().is_loopback())
    }
}
