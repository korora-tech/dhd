use super::{PackageProvider, command_exists};

pub struct PacmanProvider;

impl PackageProvider for PacmanProvider {
    fn is_available(&self) -> bool {
        command_exists("pacman")
    }
    
    fn is_package_installed(&self, package: &str) -> Result<bool, String> {
        use std::process::Command;
        
        let output = Command::new("pacman")
            .arg("-Q")
            .arg(package)
            .output()
            .map_err(|e| format!("Failed to run pacman -Q: {}", e))?;
        
        // pacman -Q returns 0 if package is installed, 1 if not
        Ok(output.status.success())
    }
    
    fn install_package(&self, package: &str) -> Result<(), String> {
        use std::process::Command;
        
        let mut cmd = Command::new("pkexec");
        cmd.arg("pacman")
            .arg("-S")
            .arg("--noconfirm")
            .arg(package);
            
        let output = cmd.output()
            .map_err(|e| format!("Failed to run pacman install: {}", e))?;
        
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Failed to install package {}: {}", package, stderr))
        }
    }
    
    fn uninstall_package(&self, _package: &str) -> Result<(), String> {
        todo!("Implement pacman uninstall")
    }
    
    fn update(&self) -> Result<(), String> {
        todo!("Implement pacman update")
    }
    
    fn name(&self) -> &str {
        "pacman"
    }
    
    fn install_command(&self) -> Vec<String> {
        vec!["pacman".to_string(), "-S".to_string(), "--noconfirm".to_string()]
    }
}