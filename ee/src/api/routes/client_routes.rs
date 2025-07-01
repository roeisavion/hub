use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use sqlx::types::Uuid;

use crate::{
    dto::{ClientResponse, CreateClientRequest, UpdateClientRequest},
    errors::ApiError,
    services::client_service::ClientService,
    AppState,
};

/// Creates the Axum router for client CRUD operations.
pub fn client_routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_client_handler).get(list_clients_handler))
        .route(
            "/:id",
            get(get_client_handler)
                .put(update_client_handler)
                .delete(delete_client_handler),
        )
}

#[utoipa::path(
    post,
    path = "/ee/api/v1/clients",
    request_body = CreateClientRequest,
    responses(
        (status = 201, description = "Client created successfully", body = ClientResponse),
        (status = 400, description = "Invalid request", body = ApiError),
        (status = 409, description = "Conflict - client name or API key already exists", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "Clients"
)]
#[axum::debug_handler]
async fn create_client_handler(
    State(app_state): State<AppState>,
    Json(payload): Json<CreateClientRequest>,
) -> Result<(StatusCode, Json<ClientResponse>), ApiError> {
    let service = ClientService::new(app_state.db_pool.clone());
    let client_response = service.create_client(payload).await?;
    Ok((StatusCode::CREATED, Json(client_response)))
}

#[utoipa::path(
    get,
    path = "/ee/api/v1/clients",
    responses(
        (status = 200, description = "List of clients", body = Vec<ClientResponse>),
        (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "Clients"
)]
#[axum::debug_handler]
async fn list_clients_handler(
    State(app_state): State<AppState>,
) -> Result<(StatusCode, Json<Vec<ClientResponse>>), ApiError> {
    let service = ClientService::new(app_state.db_pool.clone());
    let client_responses = service.list_clients().await?;
    Ok((StatusCode::OK, Json(client_responses)))
}

#[utoipa::path(
    get,
    path = "/ee/api/v1/clients/{id}",
    params(
        ("id" = Uuid, Path, description = "Client ID")
    ),
    responses(
        (status = 200, description = "Client found", body = ClientResponse),
        (status = 404, description = "Client not found", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "Clients"
)]
#[axum::debug_handler]
async fn get_client_handler(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ClientResponse>, ApiError> {
    let service = ClientService::new(app_state.db_pool.clone());
    let client_response = service.get_client(id).await?;
    Ok(Json(client_response))
}

#[utoipa::path(
    put,
    path = "/ee/api/v1/clients/{id}",
    request_body = UpdateClientRequest,
    params(
        ("id" = Uuid, Path, description = "Client ID")
    ),
    responses(
        (status = 200, description = "Client updated successfully", body = ClientResponse),
        (status = 400, description = "Invalid request", body = ApiError),
        (status = 404, description = "Client not found", body = ApiError),
        (status = 409, description = "Conflict - client name already exists", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "Clients"
)]
#[axum::debug_handler]
async fn update_client_handler(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateClientRequest>,
) -> Result<Json<ClientResponse>, ApiError> {
    let service = ClientService::new(app_state.db_pool.clone());
    let client_response = service.update_client(id, payload).await?;
    Ok(Json(client_response))
}

#[utoipa::path(
    delete,
    path = "/ee/api/v1/clients/{id}",
    params(
        ("id" = Uuid, Path, description = "Client ID")
    ),
    responses(
        (status = 200, description = "Client deleted successfully"),
        (status = 404, description = "Client not found", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "Clients"
)]
#[axum::debug_handler]
async fn delete_client_handler(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<(), ApiError> {
    let service = ClientService::new(app_state.db_pool.clone());
    service.delete_client(id).await
}
