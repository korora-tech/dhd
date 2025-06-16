use async_trait::async_trait;
use std::collections::HashMap;
use thiserror::Error;

pub mod onepassword;

#[derive(Debug, Error)]
pub enum SecretError {
    #[error("Secret not found: {0}")]
    NotFound(String),
    #[error("Invalid secret reference: {0}")]
    InvalidReference(String),
    #[error("Provider error: {0}")]
    ProviderError(String),
    #[error("Command failed: {0}")]
    CommandFailed(String),
}

#[async_trait]
pub trait SecretProvider: Send + Sync {
    async fn get_secret(&self, reference: &str) -> Result<String, SecretError>;
    async fn secret_exists(&self, reference: &str) -> Result<bool, SecretError>;
    fn validate_reference(&self, reference: &str) -> Result<(), SecretError>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum SecretReference {
    OnePassword(String),
    Environment(String),
    Literal(String),
}

impl SecretReference {
    pub fn parse(value: &str) -> Result<Self, SecretError> {
        if value.starts_with("op://") {
            Ok(SecretReference::OnePassword(value.to_string()))
        } else if value.starts_with("env://") {
            let env_var = value.strip_prefix("env://").unwrap();
            Ok(SecretReference::Environment(env_var.to_string()))
        } else if value.starts_with("literal://") {
            let literal = value.strip_prefix("literal://").unwrap();
            Ok(SecretReference::Literal(literal.to_string()))
        } else {
            Err(SecretError::InvalidReference(format!(
                "Secret reference must start with op://, env://, or literal://: {}",
                value
            )))
        }
    }

    pub async fn resolve(&self, provider: &dyn SecretProvider) -> Result<String, SecretError> {
        match self {
            SecretReference::OnePassword(reference) => provider.get_secret(reference).await,
            SecretReference::Environment(var_name) => {
                std::env::var(var_name).map_err(|_| SecretError::NotFound(var_name.clone()))
            }
            SecretReference::Literal(value) => Ok(value.clone()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            SecretReference::OnePassword(s) => s,
            SecretReference::Environment(s) => s,
            SecretReference::Literal(s) => s,
        }
    }
}

pub struct SecretResolver {
    cache: HashMap<String, String>,
}

impl SecretResolver {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub async fn resolve(&mut self, reference: &str, provider: &dyn SecretProvider) -> Result<String, SecretError> {
        if let Some(cached) = self.cache.get(reference) {
            return Ok(cached.clone());
        }

        let secret_ref = SecretReference::parse(reference)?;
        let value = secret_ref.resolve(provider).await?;
        
        self.cache.insert(reference.to_string(), value.clone());
        Ok(value)
    }

    pub async fn resolve_map(&mut self, secrets: &HashMap<String, String>, provider: &dyn SecretProvider) -> Result<HashMap<String, String>, SecretError> {
        let mut resolved = HashMap::new();
        
        for (key, reference) in secrets {
            let value = self.resolve(reference, provider).await?;
            resolved.insert(key.clone(), value);
        }
        
        Ok(resolved)
    }
}