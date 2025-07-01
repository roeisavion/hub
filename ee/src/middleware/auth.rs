use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

use crate::{
    db::models::Client, services::client_service::ClientService, AppState,
};

/// Client context that gets added to request extensions
#[derive(Debug, Clone)]
pub struct ClientContext {
    pub client_id: Uuid,
    pub client_name: String,
    pub enabled: bool,
}

impl ClientContext {
    pub fn from_client(client: Client) -> Self {
        Self {
            client_id: client.id,
            client_name: client.name,
            enabled: client.enabled,
        }
    }
}

/// Authorization middleware that validates the x-traceloop-pipeline header
/// and adds client context to the request
pub async fn client_auth_middleware(
    State(app_state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract the x-traceloop-pipeline header
    let client_key = match request.headers().get("x-traceloop-pipeline") {
        Some(header_value) => match header_value.to_str() {
            Ok(key) => key,
            Err(_) => {
                tracing::warn!("Invalid x-traceloop-pipeline header format");
                return Err(StatusCode::BAD_REQUEST);
            }
        },
        None => {
            tracing::warn!("Missing x-traceloop-pipeline header");
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    // Authenticate the client
    let client_service = ClientService::new(app_state.db_pool.clone());
    let client = match client_service.authenticate_client(client_key).await {
        Ok(Some(client)) => {
            if !client.enabled {
                tracing::warn!(
                    "Client '{}' is disabled but attempted to access resources",
                    client.name
                );
                return Err(StatusCode::FORBIDDEN);
            }
            client
        }
        Ok(None) => {
            tracing::warn!("Invalid client key provided in x-traceloop-pipeline header");
            return Err(StatusCode::UNAUTHORIZED);
        }
        Err(err) => {
            tracing::error!("Error during client authentication: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Add client context to request extensions
    let client_context = ClientContext::from_client(client);
    request.extensions_mut().insert(client_context);

    // Continue with the request
    Ok(next.run(request).await)
}

// Note: Optional authentication removed since all EE resources now require client context

/// Extract client context from request extensions
/// This is used in handlers to get the authenticated client information
pub fn extract_client_context(request: &Request) -> Option<&ClientContext> {
    request.extensions().get::<ClientContext>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_context_from_client() {
        use chrono::Utc;
        use uuid::Uuid;

        let client = Client {
            id: Uuid::new_v4(),
            name: "test-client".to_string(),
            client_key: "test-client-key".to_string(),
            enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let context = ClientContext::from_client(client.clone());
        assert_eq!(context.client_id, client.id);
        assert_eq!(context.client_name, client.name);
        assert_eq!(context.enabled, client.enabled);
    }
}
