use sqlx::{types::Uuid, PgPool};
use std::sync::Arc;

use crate::{
    db::{models::Client, repositories::client_repository::ClientRepository},
    dto::{ClientResponse, CreateClientRequest, UpdateClientRequest},
    errors::ApiError,
};

#[derive(Clone)]
pub struct ClientService {
    repo: Arc<ClientRepository>,
}

impl ClientService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            repo: Arc::new(ClientRepository::new(pool)),
        }
    }

    /// Create a new client
    pub async fn create_client(
        &self,
        request: CreateClientRequest,
    ) -> Result<ClientResponse, ApiError> {
        // Check if name already exists
        if self.repo.find_by_name(&request.name).await?.is_some() {
            return Err(ApiError::Conflict(format!(
                "Client with name '{}' already exists.",
                request.name
            )));
        }

        // Check if client key already exists
        if self
            .repo
            .find_by_client_key(&request.client_key)
            .await?
            .is_some()
        {
            return Err(ApiError::Conflict(
                "A client with this client key already exists.".to_string(),
            ));
        }

        let db_client = self.repo.create(&request).await?;
        Ok(Self::map_db_client_to_response(db_client))
    }

    /// Get a client by ID
    pub async fn get_client(&self, id: Uuid) -> Result<ClientResponse, ApiError> {
        let db_client = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApiError::NotFound(format!("Client with ID {} not found.", id)))?;
        Ok(Self::map_db_client_to_response(db_client))
    }

    /// List all clients
    pub async fn list_clients(&self) -> Result<Vec<ClientResponse>, ApiError> {
        let db_clients = self.repo.list().await?;
        Ok(db_clients
            .into_iter()
            .map(Self::map_db_client_to_response)
            .collect())
    }

    /// Update a client
    pub async fn update_client(
        &self,
        id: Uuid,
        request: UpdateClientRequest,
    ) -> Result<ClientResponse, ApiError> {
        // Check if new name conflicts with existing clients (if name is being updated)
        if let Some(ref new_name) = request.name {
            if let Some(existing_client) = self.repo.find_by_name(new_name).await? {
                if existing_client.id != id {
                    return Err(ApiError::Conflict(format!(
                        "Client with name '{}' already exists.",
                        new_name
                    )));
                }
            }
        }

        let db_client = self
            .repo
            .update(id, &request)
            .await?
            .ok_or_else(|| ApiError::NotFound(format!("Client with ID {} not found.", id)))?;
        Ok(Self::map_db_client_to_response(db_client))
    }

    /// Delete a client
    pub async fn delete_client(&self, id: Uuid) -> Result<(), ApiError> {
        let deleted = self.repo.delete(id).await?;
        if !deleted {
            return Err(ApiError::NotFound(format!(
                "Client with ID {} not found.",
                id
            )));
        }
        Ok(())
    }

    /// Authenticate a client by client key and return client info
    pub async fn authenticate_client(&self, client_key: &str) -> Result<Option<Client>, ApiError> {
        Ok(self.repo.find_by_client_key(client_key).await?)
    }

    /// Convert database client to response DTO
    fn map_db_client_to_response(db_client: Client) -> ClientResponse {
        ClientResponse {
            id: db_client.id,
            name: db_client.name,
            enabled: db_client.enabled,
            created_at: db_client.created_at,
            updated_at: db_client.updated_at,
        }
    }
}

// Tests removed since we no longer hash API keys
