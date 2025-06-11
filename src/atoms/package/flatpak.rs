use super::{PackageProvider, command_exists};
use std::process::Command;

pub struct FlatpakProvider;

impl PackageProvider for FlatpakProvider {
    fn is_available(&self) -> bool {
        command_exists("flatpak")
    }

    fn is_package_installed(&self, package: &str) -> Result<bool, String> {
        let output = Command::new("flatpak")
            .args(&["list", "--app", "--columns=application"])
            .output()
            .map_err(|e| format!("Failed to check package status: {}", e))?;

        let installed = String::from_utf8_lossy(&output.stdout);
        Ok(installed.lines().any(|line| line.trim() == package))
    }

    fn install_package(&self, package: &str) -> Result<(), String> {
        let output = Command::new("flatpak")
            .args(&["install", "-y", "flathub", package])
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
        let output = Command::new("flatpak")
            .args(&["uninstall", "-y", package])
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
        let output = Command::new("flatpak")
            .args(&["update", "-y"])
            .output()
            .map_err(|e| format!("Failed to update flatpak: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "Failed to update flatpak: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "flatpak"
    }

    fn install_command(&self) -> Vec<String> {
        vec![
            "flatpak".to_string(),
            "install".to_string(),
            "-y".to_string(),
            "flathub".to_string(),
        ]
    }
}
