use anyhow::{anyhow, Result};
use hub_gateway_core_types::{GatewayConfig, ModelConfig, Pipeline, PipelineType, PluginConfig, Provider};
use reqwest::{Client, HeaderMap, HeaderValue};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, error, info, warn};

use crate::{
    dto::{ApiConfigurationResponse, ApiModelDefinitionResponse, ApiPipelineResponseDto, ApiProviderResponse, ModelRouterConfigDto, PipelinePluginConfigDto, ProviderConfig as ApiProviderConfig},
    secret_resolver::SecretResolver,
};

#[derive(Debug, Clone)]
pub struct ApiClientConfig {
    pub base_url: String,
    pub timeout_seconds: u64,
    pub auth_header: Option<String>,
    pub auth_value: Option<String>,
    pub providers_endpoint: Option<String>,
    pub models_endpoint: Option<String>,
    pub pipelines_endpoint: Option<String>,
    pub full_config_endpoint: Option<String>,
}

impl ApiClientConfig {
    pub fn from_env() -> Result<Self> {
        let base_url = std::env::var("API_CONFIG_BASE_URL")
            .map_err(|_| anyhow!("API_CONFIG_BASE_URL environment variable is required"))?;
        let timeout_seconds = std::env::var("API_CONFIG_TIMEOUT_SECONDS")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .unwrap_or(30);
        let auth_header = std::env::var("API_CONFIG_AUTH_HEADER").ok();
        let auth_value = std::env::var("API_CONFIG_AUTH_VALUE").ok();
        let providers_endpoint = std::env::var("API_CONFIG_PROVIDERS_ENDPOINT").ok();
        let models_endpoint = std::env::var("API_CONFIG_MODELS_ENDPOINT").ok();
        let pipelines_endpoint = std::env::var("API_CONFIG_PIPELINES_ENDPOINT").ok();
        let full_config_endpoint = std::env::var("API_CONFIG_FULL_ENDPOINT").ok();

        Ok(Self {
            base_url, timeout_seconds, auth_header, auth_value,
            providers_endpoint, models_endpoint, pipelines_endpoint, full_config_endpoint,
        })
    }
}

pub struct ApiConfigProviderService {
    client: Client,
    config: ApiClientConfig,
    secret_resolver: SecretResolver,
}

impl ApiConfigProviderService {
    pub fn new(config: ApiClientConfig) -> Result<Self> {
        let mut headers = HeaderMap::new();
        if let (Some(header_name), Some(header_value)) = (&config.auth_header, &config.auth_value) {
            let header_name = header_name.parse()
                .map_err(|e| anyhow!("Invalid auth header name '{}': {}", header_name, e))?;
            let header_value = HeaderValue::from_str(header_value)
                .map_err(|e| anyhow!("Invalid auth header value: {}", e))?;
            headers.insert(header_name, header_value);
        }
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .default_headers(headers)
            .build()
            .map_err(|e| anyhow!("Failed to create HTTP client: {}", e))?;
        Ok(Self { client, config, secret_resolver: SecretResolver::new() })
    }

    pub async fn fetch_live_config(&self) -> Result<GatewayConfig> {
        info!("Fetching live configuration from external API...");
        let api_response = if let Some(full_endpoint) = &self.config.full_config_endpoint {
            self.fetch_full_config(full_endpoint).await?
        } else {
            self.fetch_config_from_separate_endpoints().await?
        };
        self.transform_api_response_to_gateway_config(api_response).await
    }

    async fn fetch_full_config(&self, endpoint: &str) -> Result<ApiConfigurationResponse> {
        let url = if endpoint.starts_with("http") {
            endpoint.to_string()
        } else {
            format!("{}/{}", self.config.base_url.trim_end_matches('/'), endpoint.trim_start_matches('/'))
        };
        debug!("Fetching full configuration from: {}", url);
        let response = self.client.get(&url).send().await
            .map_err(|e| anyhow!("Failed to fetch configuration from {}: {}", url, e))?;
        if !response.status().is_success() {
            return Err(anyhow!("API returned error status {}: {}", response.status(), response.text().await.unwrap_or_default()));
        }
        let config: ApiConfigurationResponse = response.json().await
            .map_err(|e| anyhow!("Failed to parse API response as JSON: {}", e))?;
        Ok(config)
    }

    async fn fetch_config_from_separate_endpoints(&self) -> Result<ApiConfigurationResponse> {
        let providers_endpoint = self.config.providers_endpoint.as_deref().unwrap_or("providers");
        let models_endpoint = self.config.models_endpoint.as_deref().unwrap_or("models");
        let pipelines_endpoint = self.config.pipelines_endpoint.as_deref().unwrap_or("pipelines");
        let (providers_result, models_result, pipelines_result) = tokio::try_join!(
            self.fetch_providers(providers_endpoint),
            self.fetch_models(models_endpoint),
            self.fetch_pipelines(pipelines_endpoint)
        )?;
        Ok(ApiConfigurationResponse {
            providers: providers_result,
            models: models_result,
            pipelines: pipelines_result,
            version: None,
            last_updated: None,
        })
    }

    async fn fetch_providers(&self, endpoint: &str) -> Result<Vec<ApiProviderResponse>> {
        let url = format!("{}/{}", self.config.base_url.trim_end_matches('/'), endpoint.trim_start_matches('/'));
        debug!("Fetching providers from: {}", url);
        let response = self.client.get(&url).send().await.map_err(|e| anyhow!("Failed to fetch providers: {}", e))?;
        if !response.status().is_success() {
            return Err(anyhow!("Providers API returned error status {}", response.status()));
        }
        let providers: Vec<ApiProviderResponse> = response.json().await.map_err(|e| anyhow!("Failed to parse providers response: {}", e))?;
        Ok(providers)
    }

    async fn fetch_models(&self, endpoint: &str) -> Result<Vec<ApiModelDefinitionResponse>> {
        let url = format!("{}/{}", self.config.base_url.trim_end_matches('/'), endpoint.trim_start_matches('/'));
        debug!("Fetching models from: {}", url);
        let response = self.client.get(&url).send().await.map_err(|e| anyhow!("Failed to fetch models: {}", e))?;
        if !response.status().is_success() {
            return Err(anyhow!("Models API returned error status {}", response.status()));
        }
        let models: Vec<ApiModelDefinitionResponse> = response.json().await.map_err(|e| anyhow!("Failed to parse models response: {}", e))?;
        Ok(models)
    }

    async fn fetch_pipelines(&self, endpoint: &str) -> Result<Vec<ApiPipelineResponseDto>> {
        let url = format!("{}/{}", self.config.base_url.trim_end_matches('/'), endpoint.trim_start_matches('/'));
        debug!("Fetching pipelines from: {}", url);
        let response = self.client.get(&url).send().await.map_err(|e| anyhow!("Failed to fetch pipelines: {}", e))?;
        if !response.status().is_success() {
            return Err(anyhow!("Pipelines API returned error status {}", response.status()));
        }
        let pipelines: Vec<ApiPipelineResponseDto> = response.json().await.map_err(|e| anyhow!("Failed to parse pipelines response: {}", e))?;
        Ok(pipelines)
    }

    async fn transform_api_response_to_gateway_config(&self, api_response: ApiConfigurationResponse) -> Result<GatewayConfig> {
        let mut gateway_config = GatewayConfig::default();
        let mut provider_api_id_to_key_map: HashMap<String, String> = HashMap::new();

        for api_provider in api_response.providers.into_iter().filter(|p| p.enabled) {
            let original_api_id = api_provider.id.clone();
            match self.transform_provider_dto(api_provider).await {
                Ok(core_provider) => {
                    provider_api_id_to_key_map.insert(original_api_id, core_provider.key.clone());
                    gateway_config.providers.push(core_provider);
                }
                Err(e) => error!("Failed to transform provider: {:?}. Skipping.", e),
            }
        }

        for api_model in api_response.models.into_iter().filter(|m| m.enabled) {
            match self.transform_model_dto(api_model, &provider_api_id_to_key_map) {
                Ok(core_model) => gateway_config.models.push(core_model),
                Err(e) => error!("Failed to transform model: {:?}. Skipping.", e),
            }
        }

        for api_pipeline in api_response.pipelines.into_iter().filter(|pl| pl.enabled) {
            match Self::transform_pipeline_dto(api_pipeline) {
                Ok(core_pipeline) => gateway_config.pipelines.push(core_pipeline),
                Err(e) => error!("Failed to transform pipeline: {:?}. Skipping.", e),
            }
        }

        info!("Successfully transformed API configuration: {} providers, {} models, {} pipelines",
            gateway_config.providers.len(), gateway_config.models.len(), gateway_config.pipelines.len());

        Ok(gateway_config)
    }

    async fn transform_provider_dto(&self, dto: ApiProviderResponse) -> Result<Provider> {
        let mut params = HashMap::new();
        let api_key_from_dto = match dto.config {
            ApiProviderConfig::OpenAI(c) => {
                if let Some(org_id) = c.organization_id {
                    params.insert("organization_id".to_string(), org_id);
                }
                Some(self.secret_resolver.resolve_secret(&c.api_key).await?)
            }
            ApiProviderConfig::Anthropic(c) => {
                Some(self.secret_resolver.resolve_secret(&c.api_key).await?)
            }
        };

        Ok(Provider {
            key: dto.name,
            r#type: dto.provider_type.to_string(),
            api_key: api_key_from_dto.unwrap_or_default(),
            params,
        })
    }

    fn transform_model_dto(&self, dto: ApiModelDefinitionResponse, provider_api_id_to_key_map: &HashMap<String, String>) -> Result<ModelConfig> {
        let provider_key = provider_api_id_to_key_map
            .get(&dto.provider_id)
            .ok_or_else(|| anyhow!("Provider key not found for provider ID {} (model key '{}')", dto.provider_id, dto.key))?
            .clone();

        let mut params = HashMap::new();
        match dto.config_details {
            JsonValue::Object(map) => {
                for (k, v) in map {
                    params.insert(k, self.convert_json_value_to_string(&v));
                }
            }
            JsonValue::Null => {}
            _ => warn!("Model '{}' config_details is not a JSON object.", dto.key),
        }

        Ok(ModelConfig {
            key: dto.key,
            r#type: dto.model_type,
            provider: provider_key,
            params,
        })
    }

    fn transform_pipeline_dto(dto: ApiPipelineResponseDto) -> Result<Pipeline> {
        let core_pipeline_type = match dto.pipeline_type.to_lowercase().as_str() {
            "chat" => PipelineType::Chat,
            "completion" => PipelineType::Completion,
            "embeddings" => PipelineType::Embeddings,
            _ => return Err(anyhow!("Unsupported pipeline type: {}", dto.pipeline_type)),
        };

        let mut core_plugins = Vec::new();
        for plugin_dto in dto.plugins.into_iter().filter(|p| p.enabled) {
            match Self::transform_plugin_dto(plugin_dto) {
                Ok(p) => core_plugins.push(p),
                Err(e) => error!("Failed to transform plugin DTO: {:?}. Skipping.", e),
            }
        }

        Ok(Pipeline {
            name: dto.name,
            r#type: core_pipeline_type,
            plugins: core_plugins,
        })
    }

    fn transform_plugin_dto(dto: PipelinePluginConfigDto) -> Result<PluginConfig> {
        match dto.plugin_type {
            crate::dto::PluginType::ModelRouter => {
                let mr_config: ModelRouterConfigDto = serde_json::from_value(dto.config_data)
                    .map_err(|e| anyhow!("Failed to deserialize ModelRouterConfigDto: {}", e))?;
                let model_keys = mr_config.models.into_iter().map(|m| m.key).collect();
                Ok(PluginConfig::ModelRouter { models: model_keys })
            }
            crate::dto::PluginType::Logging => {
                let level = dto.config_data.get("level").and_then(|v| v.as_str()).unwrap_or("warning").to_string();
                Ok(PluginConfig::Logging { level })
            }
            crate::dto::PluginType::Tracing => {
                let endpoint = dto.config_data.get("endpoint").and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing endpoint for tracing plugin"))?.to_string();
                let api_key = dto.config_data.get("api_key").and_then(|v| v.as_str()).map(String::from).unwrap_or_default();
                Ok(PluginConfig::Tracing { endpoint, api_key })
            }
        }
    }

    fn convert_json_value_to_string(&self, json_value: &JsonValue) -> String {
        match json_value {
            JsonValue::String(s) => s.clone(),
            JsonValue::Number(n) => n.to_string(),
            JsonValue::Bool(b) => b.to_string(),
            JsonValue::Null => String::new(),
            JsonValue::Array(_) | JsonValue::Object(_) => {
                serde_json::to_string(json_value).unwrap_or_else(|e| {
                    warn!("Failed to serialize complex JsonValue to string: {}. Using empty string.", e);
                    String::new()
                })
            }
        }
    }
}