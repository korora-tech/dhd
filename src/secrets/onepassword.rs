use super::{SecretError, SecretProvider};
use async_trait::async_trait;
use std::process::Stdio;
use tokio::process::Command;

pub struct OnePasswordProvider {
    account: Option<String>,
}

impl OnePasswordProvider {
    pub fn new(account: Option<String>) -> Self {
        Self { account }
    }

    async fn run_op_command(&self, args: &[&str]) -> Result<String, SecretError> {
        let mut cmd = Command::new("op");
        cmd.args(args);
        
        if let Some(account) = &self.account {
            cmd.arg("--account").arg(account);
        }
        
        cmd.stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let output = cmd.output().await.map_err(|e| {
            SecretError::ProviderError(format!("Failed to execute op command: {}", e))
        })?;

        if output.status.success() {
            String::from_utf8(output.stdout)
                .map_err(|e| SecretError::ProviderError(format!("Invalid UTF-8 in output: {}", e)))
                .map(|s| s.trim().to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(SecretError::CommandFailed(format!(
                "op command failed: {}",
                stderr.trim()
            )))
        }
    }

    fn parse_reference<'a>(&self, reference: &'a str) -> Result<(&'a str, &'a str, &'a str), SecretError> {
        if !reference.starts_with("op://") {
            return Err(SecretError::InvalidReference(format!(
                "Not a 1Password reference: {}",
                reference
            )));
        }

        let path = reference.strip_prefix("op://").unwrap();
        let parts: Vec<&str> = path.splitn(3, '/').collect();
        
        if parts.len() != 3 {
            return Err(SecretError::InvalidReference(format!(
                "Invalid 1Password reference format. Expected op://vault/item/field: {}",
                reference
            )));
        }

        Ok((parts[0], parts[1], parts[2]))
    }
}

#[async_trait]
impl SecretProvider for OnePasswordProvider {
    async fn get_secret(&self, reference: &str) -> Result<String, SecretError> {
        self.validate_reference(reference)?;
        
        let args = vec!["read", reference];
        self.run_op_command(&args).await
    }

    async fn secret_exists(&self, reference: &str) -> Result<bool, SecretError> {
        self.validate_reference(reference)?;
        
        let (vault, item, _) = self.parse_reference(reference)?;
        let item_reference = format!("op://{}/{}", vault, item);
        
        let args = vec!["item", "get", &item_reference, "--format", "json"];
        
        match self.run_op_command(&args).await {
            Ok(_) => Ok(true),
            Err(SecretError::CommandFailed(msg)) if msg.contains("isn't an item") => Ok(false),
            Err(e) => Err(e),
        }
    }

    fn validate_reference(&self, reference: &str) -> Result<(), SecretError> {
        self.parse_reference(reference)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_reference() {
        let provider = OnePasswordProvider::new(None);
        
        assert!(provider.parse_reference("op://Personal/GitHub/token").is_ok());
        assert!(provider.parse_reference("op://Work/Database/password").is_ok());
        
        assert!(provider.parse_reference("not-op-reference").is_err());
        assert!(provider.parse_reference("op://incomplete").is_err());
        assert!(provider.parse_reference("op://only/two").is_err());
    }
}