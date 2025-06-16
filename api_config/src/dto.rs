use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, ToSchema, Clone, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum SecretObject {
    #[serde(rename = "literal")]
    Literal {
        value: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        encrypted: Option<bool>,
    },
    #[serde(rename = "kubernetes")]
    Kubernetes {
        secret_name: String,
        key: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        namespace: Option<String>,
    },
    #[serde(rename = "environment")]
    Environment { variable_name: String },
}

impl SecretObject {
    pub fn literal(value: String) -> Self {
        Self::Literal { value, encrypted: None }
    }
    pub fn environment(variable_name: String) -> Self {
        Self::Environment { variable_name }
    }
}

#[derive(Serialize, Deserialize, Debug, ToSchema, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ProviderType {
    Azure, OpenAI, Anthropic, Bedrock, VertexAI,
}

impl std::fmt::Display for ProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ProviderType::Azure => write!(f, "azure"),
            ProviderType::OpenAI => write!(f, "openai"),
            ProviderType::Anthropic => write!(f, "anthropic"),
            ProviderType::Bedrock => write!(f, "bedrock"),
            ProviderType::VertexAI => write!(f, "vertexai"),
        }
    }
}
#[derive(Serialize, Deserialize, Debug, ToSchema, Clone, PartialEq, Eq)]
pub struct OpenAIProviderConfig {
    pub api_key: SecretObject,
    pub organization_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema, Clone, PartialEq, Eq)]
pub struct AnthropicProviderConfig {
    pub api_key: SecretObject,
}

#[derive(Serialize, Deserialize, Debug, ToSchema, Clone, PartialEq)]
#[serde(untagged)]
pub enum ProviderConfig {
    OpenAI(OpenAIProviderConfig),
    Anthropic(AnthropicProviderConfig),
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq, Clone)]
pub struct ApiProviderResponse {
    pub id: String,
    pub name: String,
    pub provider_type: ProviderType,
    pub config: ProviderConfig,
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct ApiModelDefinitionResponse {
    pub id: String,
    pub key: String,
    pub model_type: String,
    pub provider_id: String,
    pub config_details: serde_json::Value,
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, PartialEq)]
pub struct ModelRouterModelEntryDto {
    pub key: String,
    pub priority: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, PartialEq)]
pub struct ModelRouterConfigDto {
    pub models: Vec<ModelRouterModelEntryDto>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum PluginType {
    ModelRouter,
    Logging,
    Tracing,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, PartialEq)]
pub struct PipelinePluginConfigDto {
    pub plugin_type: PluginType,
    pub config_data: serde_json::Value,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub order_in_pipeline: i32,
}

fn default_true() -> bool { true }

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct ApiPipelineResponseDto {
    pub id: String,
    pub name: String,
    pub pipeline_type: String,
    pub description: Option<String>,
    pub plugins: Vec<PipelinePluginConfigDto>,
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct ApiConfigurationResponse {
    pub providers: Vec<ApiProviderResponse>,
    pub models: Vec<ApiModelDefinitionResponse>,
    pub pipelines: Vec<ApiPipelineResponseDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<DateTime<Utc>>,
}