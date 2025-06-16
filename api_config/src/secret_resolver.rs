use anyhow::{anyhow, Result};
use std::env;
use crate::dto::SecretObject;

pub struct SecretResolver {}

impl SecretResolver {
    pub fn new() -> Self { Self {} }
    
    pub async fn resolve_secret(&self, secret: &SecretObject) -> Result<String> {
        match secret {
            SecretObject::Literal { value, .. } => Ok(value.clone()),
            SecretObject::Environment { variable_name } => {
                env::var(variable_name).map_err(|e| anyhow!("Failed to resolve env var: {}", e))
            }
            SecretObject::Kubernetes { .. } => {
                Err(anyhow!("Kubernetes secrets not implemented"))
            }
        }
    }
}

impl Default for SecretResolver {
    fn default() -> Self { Self::new() }
}