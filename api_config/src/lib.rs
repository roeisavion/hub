pub mod config_provider_service;
pub mod dto;
pub mod secret_resolver;

pub use config_provider_service::{ApiClientConfig, ApiConfigProviderService};

use anyhow::Result;
use std::sync::Arc;

/// Integration result containing the API config provider service
pub struct ApiConfigIntegration {
    pub config_provider: Arc<ApiConfigProviderService>,
}

/// Initializes the API-based configuration system
pub async fn api_config_integration() -> Result<ApiConfigIntegration> {
    let client_config = ApiClientConfig::from_env()?;
    let config_provider = Arc::new(ApiConfigProviderService::new(client_config)?);

    Ok(ApiConfigIntegration { config_provider })
}