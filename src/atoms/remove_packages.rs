use crate::atoms::Atom;
use crate::atoms::package::PackageManager;
use crate::platform::{LinuxDistro, Platform, current_platform};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct RemovePackages {
    pub names: Vec<String>,
    pub manager: Option<PackageManager>,
}

impl RemovePackages {
    pub fn new(names: Vec<String>, manager: Option<PackageManager>) -> Self {
        Self { names, manager }
    }

    fn detect_package_manager(&self) -> Result<PackageManager, String> {
        if let Some(manager) = &self.manager {
            return Ok(manager.clone());
        }

        match current_platform() {
            Platform::Linux(distro) => match distro {
                LinuxDistro::Ubuntu | LinuxDistro::Debian => Ok(PackageManager::Apt),
                LinuxDistro::Fedora => Ok(PackageManager::Dnf),
                LinuxDistro::Arch => Ok(PackageManager::Pacman),
                LinuxDistro::NixOS => Ok(PackageManager::Nix),
                _ => {
                    Err("Unable to detect package manager for this Linux distribution".to_string())
                }
            },
            Platform::MacOS => Ok(PackageManager::Brew),
            _ => Err("Unsupported platform for package removal".to_string()),
        }
    }
}

impl Atom for RemovePackages {
    fn name(&self) -> &str {
        "RemovePackages"
    }

    fn execute(&self) -> Result<(), String> {
        if self.names.is_empty() {
            return Ok(());
        }

        let manager = self.detect_package_manager()?;

        let (command, args) = match manager {
            PackageManager::Apt => ("sudo", vec!["apt-get", "remove", "-y"]),
            PackageManager::Dnf => ("sudo", vec!["dnf", "remove", "-y"]),
            PackageManager::Pacman => ("sudo", vec!["pacman", "-R", "--noconfirm"]),
            PackageManager::Brew => ("brew", vec!["uninstall"]),
            PackageManager::Nix => return Err("Package removal not supported for Nix".to_string()),
            _ => return Err(format!("Package removal not implemented for {:?}", manager)),
        };

        let mut cmd = Command::new(command);
        for arg in args {
            cmd.arg(arg);
        }
        for package in &self.names {
            cmd.arg(package);
        }

        let output = cmd
            .output()
            .map_err(|e| format!("Failed to execute package removal: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Check if packages are not installed (not an error)
            if stderr.contains("is not installed") || stderr.contains("No packages found") {
                println!("Some packages were not installed, skipping");
                return Ok(());
            }
            return Err(format!("Failed to remove packages: {}", stderr));
        }

        Ok(())
    }

    fn describe(&self) -> String {
        format!("Remove packages: {}", self.names.join(", "))
    }
}
