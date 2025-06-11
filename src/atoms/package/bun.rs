use super::{PackageProvider, command_exists};
use std::process::Command;

pub struct BunProvider;

impl PackageProvider for BunProvider {
    fn is_available(&self) -> bool {
        command_exists("bun")
    }

    fn is_package_installed(&self, package: &str) -> Result<bool, String> {
        // For global packages, check if the command exists
        // For local packages, we'd need to check package.json
        // This is a simplified implementation
        if package.starts_with('@') || package.contains('/') {
            // Check if it's a global package by trying to find its binary
            let binary_name = package.split('/').last().unwrap_or(package);
            Ok(command_exists(binary_name))
        } else {
            Ok(command_exists(package))
        }
    }

    fn install_package(&self, package: &str) -> Result<(), String> {
        let output = Command::new("bun")
            .args(&["add", "--global", package])
            .output()
            .map_err(|e| format!("Failed to install package: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "Failed to install {}: {}",
                package,
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(())
    }

    fn uninstall_package(&self, package: &str) -> Result<(), String> {
        let output = Command::new("bun")
            .args(&["remove", "--global", package])
            .output()
            .map_err(|e| format!("Failed to uninstall package: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "Failed to uninstall {}: {}",
                package,
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(())
    }

    fn update(&self) -> Result<(), String> {
        // Bun updates itself
        let output = Command::new("bun")
            .args(&["upgrade"])
            .output()
            .map_err(|e| format!("Failed to update bun: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "Failed to update bun: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "bun"
    }

    fn install_command(&self) -> Vec<String> {
        vec!["bun".to_string(), "add".to_string(), "--global".to_string()]
    }
}
