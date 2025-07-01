use serde_json::Value as JsonValue;
use sqlx::{query, query_as, types::Uuid, PgPool, Result as SqlxResult};

use crate::db::models::Provider;
use crate::dto::{CreateProviderRequest, UpdateProviderRequest}; // Using DTOs

// We might need CreateProviderRequest or similar DTOs if we pass parts of them directly,
// or we pass decomposed values (name, provider_type string, config_details JsonValue).

// For now, let's assume a simplified ProviderData struct or individual params for creation/update.

#[derive(Debug)] // Temporary struct for conveying data, replace with actual DTOs or decomposed params
pub struct CreateProviderData {
    pub name: String,
    pub provider_type: String, // as string, matching DB
    pub config_details: JsonValue,
    pub enabled: bool,
}

#[derive(Debug)] // Temporary struct
pub struct UpdateProviderData {
    pub name: Option<String>,
    pub config_details: Option<JsonValue>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone)] // Added Clone
pub struct ProviderRepository {
    pool: PgPool,
}

impl ProviderRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        data: &CreateProviderRequest,
        provider_type_str: &str,
        config_json_value: JsonValue,
        client_id: Uuid,
    ) -> SqlxResult<Provider> {
        let new_id = Uuid::new_v4();
        let enabled = data.enabled.unwrap_or(true);
        query_as!(
            Provider,
            r#"
            INSERT INTO hub_llmgateway_ee_providers (id, name, provider_type, config_details, enabled, client_id)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, name, provider_type, config_details, enabled, client_id, created_at, updated_at
            "#,
            new_id,
            data.name,
            provider_type_str,
            config_json_value,
            enabled,
            client_id
        )
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_by_id(&self, id: Uuid, client_id: Uuid) -> SqlxResult<Option<Provider>> {
        query_as!(
            Provider,
            r#"
            SELECT id, name, provider_type, config_details, enabled, client_id, created_at, updated_at
            FROM hub_llmgateway_ee_providers
            WHERE id = $1 AND client_id = $2
            "#,
            id,
            client_id
        )
        .fetch_optional(&self.pool)
        .await
    }

    /// Finds a provider by ID without client restriction - used for system-level operations
    pub async fn find_by_id_system_level(&self, id: Uuid) -> SqlxResult<Option<Provider>> {
        query_as!(
            Provider,
            r#"
            SELECT id, name, provider_type, config_details, enabled, client_id, created_at, updated_at
            FROM hub_llmgateway_ee_providers
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_by_name(&self, name: &str, client_id: Uuid) -> SqlxResult<Option<Provider>> {
        query_as!(
            Provider,
            r#"
            SELECT id, name, provider_type, config_details, enabled, client_id, created_at, updated_at
            FROM hub_llmgateway_ee_providers
            WHERE name = $1 AND client_id = $2
            "#,
            name,
            client_id
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list(&self, client_id: Uuid) -> SqlxResult<Vec<Provider>> {
        query_as!(
            Provider,
            r#"
            SELECT id, name, provider_type, config_details, enabled, client_id, created_at, updated_at
            FROM hub_llmgateway_ee_providers
            WHERE client_id = $1
            ORDER BY name
            "#,
            client_id
        )
        .fetch_all(&self.pool)
        .await
    }

    /// Lists all providers across all clients - used for system-level operations
    pub async fn list_all(&self) -> SqlxResult<Vec<Provider>> {
        query_as!(
            Provider,
            r#"
            SELECT id, name, provider_type, config_details, enabled, client_id, created_at, updated_at
            FROM hub_llmgateway_ee_providers
            ORDER BY name
            "#
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn update(
        &self,
        id: Uuid,
        data: &UpdateProviderRequest,
        config_json_value_opt: Option<JsonValue>,
        client_id: Uuid,
    ) -> SqlxResult<Option<Provider>> {
        let current_provider = self
            .find_by_id(id, client_id)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let name_to_update = data.name.as_ref().unwrap_or(&current_provider.name);
        let enabled_to_update = data.enabled.unwrap_or(current_provider.enabled);

        let final_config_details: JsonValue = match config_json_value_opt {
            Some(new_val) => new_val,
            None => current_provider.config_details.clone(),
        };

        query_as!(
            Provider,
            r#"
            UPDATE hub_llmgateway_ee_providers
            SET
                name = $1,
                config_details = $2,
                enabled = $3,
                updated_at = now()
            WHERE id = $4 AND client_id = $5
            RETURNING id, name, provider_type, config_details, enabled, client_id, created_at, updated_at
            "#,
            name_to_update,
            final_config_details,
            enabled_to_update,
            id,
            client_id
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn delete(&self, id: Uuid, client_id: Uuid) -> SqlxResult<u64> {
        let result = query!(
            r#"
            DELETE FROM hub_llmgateway_ee_providers
            WHERE id = $1 AND client_id = $2
            "#,
            id,
            client_id
        )
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }
}
