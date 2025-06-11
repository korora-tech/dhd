use super::{PackageProvider, command_exists};
use std::process::Command;

pub struct CargoProvider;

impl PackageProvider for CargoProvider {
    fn is_available(&self) -> bool {
        command_exists("cargo")
    }

    fn is_package_installed(&self, package: &str) -> Result<bool, String> {
        // Check if the binary exists (cargo installs to ~/.cargo/bin)
        // This is a simplified check - ideally we'd parse cargo install --list
        let home = std::env::var("HOME").map_err(|_| "HOME not set")?;
        let cargo_bin = format!("{}/.cargo/bin/{}", home, package);
        Ok(std::path::Path::new(&cargo_bin).exists())
    }

    fn install_package(&self, package: &str) -> Result<(), String> {
        let output = Command::new("cargo")
            .args(&["install", package])
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
        let output = Command::new("cargo")
            .args(&["uninstall", package])
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
        // Cargo doesn't have a direct update command for the registry
        // Individual packages can be updated with cargo install --force
        Ok(())
    }

    fn name(&self) -> &str {
        "cargo"
    }

    fn install_command(&self) -> Vec<String> {
        vec!["cargo".to_string(), "install".to_string()]
    }
}
