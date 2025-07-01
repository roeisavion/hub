use crate::{
    db::models::ModelDefinition,
    dto::{CreateModelDefinitionRequest, UpdateModelDefinitionRequest},
};
use sqlx::{query, query_as, types::Uuid, PgPool, Result};

#[derive(Debug, Clone)]
pub struct ModelDefinitionRepository {
    pool: PgPool,
}

impl ModelDefinitionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        data: &CreateModelDefinitionRequest,
        client_id: Uuid,
    ) -> Result<ModelDefinition> {
        let default_enabled = data.enabled.unwrap_or(true);
        let model_def = query_as!(ModelDefinition,
            r#"
            INSERT INTO hub_llmgateway_ee_model_definitions (key, model_type, provider_id, config_details, enabled, client_id)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, key, model_type, provider_id, config_details, enabled, client_id, created_at, updated_at
            "#,
            data.key,
            data.model_type,
            data.provider_id,
            data.config_details.as_ref().map(|val| val.clone()), // Option<Value> -> Option<Value>
            default_enabled,
            client_id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(model_def)
    }

    pub async fn find_by_id(&self, id: Uuid, client_id: Uuid) -> Result<Option<ModelDefinition>> {
        query_as!(ModelDefinition,
            "SELECT id, key, model_type, provider_id, config_details, enabled, client_id, created_at, updated_at FROM hub_llmgateway_ee_model_definitions WHERE id = $1 AND client_id = $2",
            id, client_id
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_by_key(&self, key: &str, client_id: Uuid) -> Result<Option<ModelDefinition>> {
        query_as!(ModelDefinition,
            "SELECT id, key, model_type, provider_id, config_details, enabled, client_id, created_at, updated_at FROM hub_llmgateway_ee_model_definitions WHERE key = $1 AND client_id = $2",
            key, client_id
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list(&self, client_id: Uuid) -> Result<Vec<ModelDefinition>> {
        query_as!(ModelDefinition, "SELECT id, key, model_type, provider_id, config_details, enabled, client_id, created_at, updated_at FROM hub_llmgateway_ee_model_definitions WHERE client_id = $1 ORDER BY key ASC", client_id)
            .fetch_all(&self.pool)
            .await
    }

    /// Lists all model definitions across all clients - used for system-level operations
    pub async fn list_all(&self) -> Result<Vec<ModelDefinition>> {
        query_as!(ModelDefinition, "SELECT id, key, model_type, provider_id, config_details, enabled, client_id, created_at, updated_at FROM hub_llmgateway_ee_model_definitions ORDER BY key ASC")
            .fetch_all(&self.pool)
            .await
    }

    pub async fn update(
        &self,
        id: Uuid,
        data: &UpdateModelDefinitionRequest,
        client_id: Uuid,
    ) -> Result<ModelDefinition> {
        // Fetch current to handle Option fields correctly
        let current_model = self
            .find_by_id(id, client_id)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let key = data.key.as_ref().unwrap_or(&current_model.key);
        let model_type = data
            .model_type
            .as_ref()
            .unwrap_or(&current_model.model_type);
        let provider_id = data.provider_id.unwrap_or(current_model.provider_id);
        let enabled = data.enabled.unwrap_or(current_model.enabled);

        // For config_details, if Option is Some(Value), update. If Some(Null), set to NULL. If None, keep current.
        let config_details_to_update = match data.config_details.as_ref() {
            Some(serde_json::Value::Null) => None, // Explicitly set to NULL
            Some(value) => Some(value.clone()),    // Update with new value
            None => current_model.config_details.clone(), // Keep existing value
        };

        let model_def = query_as!(ModelDefinition,
            r#"
            UPDATE hub_llmgateway_ee_model_definitions
            SET key = $1, model_type = $2, provider_id = $3, config_details = $4, enabled = $5, updated_at = NOW()
            WHERE id = $6 AND client_id = $7
            RETURNING id, key, model_type, provider_id, config_details, enabled, client_id, created_at, updated_at
            "#,
            key,
            model_type,
            provider_id,
            config_details_to_update,
            enabled,
            id,
            client_id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(model_def)
    }

    pub async fn delete(&self, id: Uuid, client_id: Uuid) -> Result<u64> {
        let result = query!(
            "DELETE FROM hub_llmgateway_ee_model_definitions WHERE id = $1 AND client_id = $2",
            id,
            client_id
        )
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }
}
