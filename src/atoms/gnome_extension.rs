use crate::atoms::Atom;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct InstallGnomeExtension {
    pub extension_id: String,
}

impl InstallGnomeExtension {
    pub fn new(extension_id: String) -> Self {
        Self { extension_id }
    }
}

impl Atom for InstallGnomeExtension {
    fn name(&self) -> &str {
        "InstallGnomeExtension"
    }

    fn execute(&self) -> Result<(), String> {
        // Check if gnome-extensions-cli is installed
        let check = Command::new("which")
            .arg("gext")
            .output()
            .map_err(|e| format!("Failed to check for gext: {}", e))?;

        if !check.status.success() {
            return Err(
                "gnome-extensions-cli (gext) is not installed. Please install it first."
                    .to_string(),
            );
        }

        // Install the extension
        let output = Command::new("gext")
            .args(["install", &self.extension_id])
            .output()
            .map_err(|e| format!("Failed to execute gext install: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Check if already installed
            if stderr.contains("already installed") {
                println!("Extension {} is already installed", self.extension_id);
                return Ok(());
            }
            return Err(format!(
                "Failed to install GNOME extension {}: {}",
                self.extension_id, stderr
            ));
        }

        // Enable the extension
        let enable_output = Command::new("gext")
            .args(["enable", &self.extension_id])
            .output()
            .map_err(|e| format!("Failed to execute gext enable: {}", e))?;

        if !enable_output.status.success() {
            eprintln!(
                "Warning: Extension {} installed but could not be enabled: {}",
                self.extension_id,
                String::from_utf8_lossy(&enable_output.stderr)
            );
        }

        Ok(())
    }

    fn describe(&self) -> String {
        format!("Install GNOME extension: {}", self.extension_id)
    }
}
