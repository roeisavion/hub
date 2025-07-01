use crate::{
    db::models::ModelDefinition,
    db::repositories::{
        model_definition_repository::ModelDefinitionRepository,
        provider_repository::ProviderRepository,
    },
    dto::{
        CreateModelDefinitionRequest, ModelDefinitionResponse, ProviderResponse, ProviderType,
        UpdateModelDefinitionRequest,
    },
    errors::ApiError,
};
use sqlx::{types::Uuid, PgPool};
use std::sync::Arc;

#[derive(Clone)]
pub struct ModelDefinitionService {
    repo: Arc<ModelDefinitionRepository>,
    provider_repo: Arc<ProviderRepository>, // To fetch provider details
}

impl ModelDefinitionService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            repo: Arc::new(ModelDefinitionRepository::new(pool.clone())),
            provider_repo: Arc::new(ProviderRepository::new(pool)),
        }
    }

    async fn map_db_model_to_response(
        &self,
        db_model: ModelDefinition,
        client_id: Uuid,
    ) -> Result<ModelDefinitionResponse, ApiError> {
        // 1. Fetch provider (must belong to same client)
        let provider_db = self
            .provider_repo
            .find_by_id(db_model.provider_id, client_id)
            .await?
            .ok_or_else(|| {
                ApiError::InternalServerError(format!(
                    "Consistency error: Provider ID {} referenced by Model Definition {} not found",
                    db_model.provider_id, db_model.id
                ))
            })?;

        // 2. Parse provider's type
        let provider_type_enum: ProviderType = provider_db.provider_type.parse().map_err(|e| {
            ApiError::InternalServerError(format!(
                "Failed to parse provider_type '{}' from DB for provider ID {}: {}",
                provider_db.provider_type, provider_db.id, e
            ))
        })?;

        // 3. Deserialize provider's config using the provider type
        let provider_config =
            crate::services::provider_service::ProviderService::deserialize_provider_config(
                &provider_type_enum,
                &provider_db.config_details,
            )
            .map_err(|e| {
                ApiError::InternalServerError(format!(
                    "Failed to deserialize provider config for provider ID {}: {:?}",
                    provider_db.id, e
                ))
            })?;

        let provider_response = ProviderResponse {
            id: provider_db.id,
            name: provider_db.name,
            provider_type: provider_type_enum,
            config: provider_config,
            enabled: provider_db.enabled,
            created_at: provider_db.created_at,
            updated_at: provider_db.updated_at,
        };

        Ok(ModelDefinitionResponse {
            id: db_model.id,
            key: db_model.key,
            model_type: db_model.model_type,
            provider: provider_response,
            config_details: db_model.config_details.unwrap_or(serde_json::Value::Null),
            enabled: db_model.enabled,
            created_at: db_model.created_at,
            updated_at: db_model.updated_at,
        })
    }

    pub async fn create_model_definition(
        &self,
        data: CreateModelDefinitionRequest,
        client_id: Uuid,
    ) -> Result<ModelDefinitionResponse, ApiError> {
        // Check if provider_id exists and belongs to client
        if self
            .provider_repo
            .find_by_id(data.provider_id, client_id)
            .await?
            .is_none()
        {
            return Err(ApiError::ValidationError(format!(
                "Provider with ID {} does not exist or is not accessible",
                data.provider_id
            )));
        }

        // Check if key is unique within client scope
        if self.repo.find_by_key(&data.key, client_id).await?.is_some() {
            return Err(ApiError::Conflict(format!(
                "Model Definition key '{}' already exists",
                data.key
            )));
        }

        let new_db_model = self.repo.create(&data, client_id).await?;
        self.map_db_model_to_response(new_db_model, client_id).await
    }

    pub async fn get_model_definition(
        &self,
        id: Uuid,
        client_id: Uuid,
    ) -> Result<ModelDefinitionResponse, ApiError> {
        let db_model = self.repo.find_by_id(id, client_id).await?.ok_or_else(|| {
            ApiError::NotFound(format!("Model Definition with ID {} not found", id))
        })?;
        self.map_db_model_to_response(db_model, client_id).await
    }

    pub async fn get_model_definition_by_key(
        &self,
        key: String,
        client_id: Uuid,
    ) -> Result<ModelDefinitionResponse, ApiError> {
        let db_model = self
            .repo
            .find_by_key(&key, client_id)
            .await?
            .ok_or_else(|| {
                ApiError::NotFound(format!("Model Definition with key '{}' not found", key))
            })?;
        self.map_db_model_to_response(db_model, client_id).await
    }

    pub async fn list_model_definitions(
        &self,
        client_id: Uuid,
    ) -> Result<Vec<ModelDefinitionResponse>, ApiError> {
        let db_models = self.repo.list(client_id).await?;
        let mut responses = Vec::new();
        for db_model in db_models {
            responses.push(self.map_db_model_to_response(db_model, client_id).await?);
        }
        Ok(responses)
    }

    /// Lists all model definitions across all clients - used for system-level operations
    pub async fn list_all_model_definitions(&self) -> Result<Vec<ModelDefinitionResponse>, ApiError> {
        let db_models = self.repo.list_all().await?;
        let mut responses = Vec::new();
        for db_model in db_models {
            responses.push(self.map_db_model_to_response_system_level(db_model).await?);
        }
        Ok(responses)
    }

    /// Maps a model to response for system-level operations (provider from any client)
    async fn map_db_model_to_response_system_level(
        &self,
        db_model: ModelDefinition,
    ) -> Result<ModelDefinitionResponse, ApiError> {
        // For system-level access, fetch provider by ID without client restriction
        let provider_db = self
            .provider_repo
            .find_by_id_system_level(db_model.provider_id)
            .await?
            .ok_or_else(|| {
                ApiError::InternalServerError(format!(
                    "Consistency error: Provider ID {} referenced by Model Definition {} not found",
                    db_model.provider_id, db_model.id
                ))
            })?;

        // Parse provider's type
        let provider_type_enum: ProviderType = provider_db.provider_type.parse().map_err(|e| {
            ApiError::InternalServerError(format!(
                "Failed to parse provider_type '{}' from DB for provider ID {}: {}",
                provider_db.provider_type, provider_db.id, e
            ))
        })?;

        // Deserialize provider's config using the provider type
        let provider_config =
            crate::services::provider_service::ProviderService::deserialize_provider_config(
                &provider_type_enum,
                &provider_db.config_details,
            )
            .map_err(|e| {
                ApiError::InternalServerError(format!(
                    "Failed to deserialize provider config for provider ID {}: {:?}",
                    provider_db.id, e
                ))
            })?;

        let provider_response = ProviderResponse {
            id: provider_db.id,
            name: provider_db.name,
            provider_type: provider_type_enum,
            config: provider_config,
            enabled: provider_db.enabled,
            created_at: provider_db.created_at,
            updated_at: provider_db.updated_at,
        };

        Ok(ModelDefinitionResponse {
            id: db_model.id,
            key: db_model.key,
            model_type: db_model.model_type,
            provider: provider_response,
            config_details: db_model.config_details.unwrap_or(serde_json::Value::Null),
            enabled: db_model.enabled,
            created_at: db_model.created_at,
            updated_at: db_model.updated_at,
        })
    }

    pub async fn update_model_definition(
        &self,
        id: Uuid,
        data: UpdateModelDefinitionRequest,
        client_id: Uuid,
    ) -> Result<ModelDefinitionResponse, ApiError> {
        // Ensure the model definition to update exists and belongs to client
        let _ = self.repo.find_by_id(id, client_id).await?.ok_or_else(|| {
            ApiError::NotFound(format!("Model Definition with ID {} not found", id))
        })?;

        // If key is being updated, check for uniqueness within client scope
        if let Some(key) = &data.key {
            if let Some(existing_by_key) = self.repo.find_by_key(key, client_id).await? {
                if existing_by_key.id != id {
                    return Err(ApiError::Conflict(format!(
                        "Model Definition key '{}' already exists",
                        key
                    )));
                }
            }
        }

        // If provider_id is being updated, check if it exists and belongs to client
        if let Some(provider_id) = data.provider_id {
            if self
                .provider_repo
                .find_by_id(provider_id, client_id)
                .await?
                .is_none()
            {
                return Err(ApiError::ValidationError(format!(
                    "Provider with ID {} does not exist or is not accessible",
                    provider_id
                )));
            }
        }

        let updated_db_model = self.repo.update(id, &data, client_id).await?;
        self.map_db_model_to_response(updated_db_model, client_id)
            .await
    }

    pub async fn delete_model_definition(&self, id: Uuid, client_id: Uuid) -> Result<(), ApiError> {
        let rows_affected = self.repo.delete(id, client_id).await?;
        if rows_affected == 0 {
            return Err(ApiError::NotFound(format!(
                "Model Definition with ID {} not found",
                id
            )));
        }
        Ok(())
    }
}
