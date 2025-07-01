use crate::db::models::Client;
use crate::dto::{CreateClientRequest, UpdateClientRequest};
use sqlx::{query_as, PgPool, Result as SqlxResult};
use uuid::Uuid;

pub struct ClientRepository {
    pool: PgPool,
}

impl ClientRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, data: &CreateClientRequest) -> SqlxResult<Client> {
        query_as!(
            Client,
            r#"
            INSERT INTO hub_llmgateway_ee_clients (name, client_key, enabled)
            VALUES ($1, $2, $3)
            RETURNING id, name, client_key, enabled, created_at, updated_at
            "#,
            data.name,
            data.client_key,
            data.enabled
        )
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_by_id(&self, id: Uuid) -> SqlxResult<Option<Client>> {
        query_as!(
            Client,
            r#"
            SELECT id, name, client_key, enabled, created_at, updated_at
            FROM hub_llmgateway_ee_clients
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_by_name(&self, name: &str) -> SqlxResult<Option<Client>> {
        query_as!(
            Client,
            r#"
            SELECT id, name, client_key, enabled, created_at, updated_at
            FROM hub_llmgateway_ee_clients
            WHERE name = $1
            "#,
            name
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_by_client_key(&self, client_key: &str) -> SqlxResult<Option<Client>> {
        query_as!(
            Client,
            r#"
            SELECT id, name, client_key, enabled, created_at, updated_at
            FROM hub_llmgateway_ee_clients
            WHERE client_key = $1 AND enabled = true
            "#,
            client_key
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list(&self) -> SqlxResult<Vec<Client>> {
        query_as!(
            Client,
            r#"
            SELECT id, name, client_key, enabled, created_at, updated_at
            FROM hub_llmgateway_ee_clients
            ORDER BY name
            "#
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn update(&self, id: Uuid, data: &UpdateClientRequest) -> SqlxResult<Option<Client>> {
        let current_client = self
            .find_by_id(id)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let name_to_update = data.name.as_ref().unwrap_or(&current_client.name);
        let enabled_to_update = data.enabled.unwrap_or(current_client.enabled);

        query_as!(
            Client,
            r#"
            UPDATE hub_llmgateway_ee_clients
            SET name = $2, enabled = $3, updated_at = NOW()
            WHERE id = $1
            RETURNING id, name, client_key, enabled, created_at, updated_at
            "#,
            id,
            name_to_update,
            enabled_to_update
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn delete(&self, id: Uuid) -> SqlxResult<bool> {
        let result = sqlx::query!(
            r#"
            DELETE FROM hub_llmgateway_ee_clients
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
