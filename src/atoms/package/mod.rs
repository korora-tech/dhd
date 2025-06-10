use std::process::Command;
use dhd_macros::typescript_enum;

pub mod apt;
pub mod brew;
pub mod bun;
pub mod cargo;
pub mod dnf;
pub mod flatpak;
pub mod npm;
pub mod pacman;
pub mod snap;
pub mod go;
pub mod pip;

#[typescript_enum]
pub enum PackageManager {
    Apt,
    Brew,
    Bun,
    Cargo,
    Dnf,
    Flatpak,
    Npm,
    Pacman,
    Snap,
    Go,
    Yum,
    Zypper,
    Pip,
    Gem,
    Nix,
}

pub trait PackageProvider: Send + Sync {
    /// Check if this package manager is available on the system
    fn is_available(&self) -> bool;

    /// Check if a package is installed
    fn is_package_installed(&self, package: &str) -> Result<bool, String>;

    /// Install a package
    fn install_package(&self, package: &str) -> Result<(), String>;

    /// Install multiple packages
    fn install_packages(&self, packages: &[String]) -> Result<(), String> {
        for package in packages {
            self.install_package(package)?;
        }
        Ok(())
    }

    /// Uninstall a package
    fn uninstall_package(&self, package: &str) -> Result<(), String>;

    /// Update package manager cache/database
    fn update(&self) -> Result<(), String>;

    /// Get the name of this package manager
    fn name(&self) -> &str;

    /// Get the command used for installation
    fn install_command(&self) -> Vec<String>;
}

impl PackageManager {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "apt" => Some(PackageManager::Apt),
            "brew" => Some(PackageManager::Brew),
            "bun" => Some(PackageManager::Bun),
            "cargo" => Some(PackageManager::Cargo),
            "dnf" => Some(PackageManager::Dnf),
            "flatpak" => Some(PackageManager::Flatpak),
            "npm" => Some(PackageManager::Npm),
            "pacman" => Some(PackageManager::Pacman),
            "snap" => Some(PackageManager::Snap),
            "go" => Some(PackageManager::Go),
            "yum" => Some(PackageManager::Yum),
            "zypper" => Some(PackageManager::Zypper),
            "pip" => Some(PackageManager::Pip),
            "gem" => Some(PackageManager::Gem),
            "nix" => Some(PackageManager::Nix),
            _ => None,
        }
    }

    pub fn get_provider(&self) -> Box<dyn PackageProvider> {
        match self {
            PackageManager::Apt => Box::new(apt::AptProvider),
            PackageManager::Brew => Box::new(brew::BrewProvider),
            PackageManager::Bun => Box::new(bun::BunProvider),
            PackageManager::Cargo => Box::new(cargo::CargoProvider),
            PackageManager::Dnf => Box::new(dnf::DnfProvider),
            PackageManager::Flatpak => Box::new(flatpak::FlatpakProvider),
            PackageManager::Npm => Box::new(npm::NpmProvider),
            PackageManager::Pacman => Box::new(pacman::PacmanProvider),
            PackageManager::Snap => Box::new(snap::SnapProvider),
            PackageManager::Go => Box::new(go::GoProvider),
            PackageManager::Pip => Box::new(pip::PipProvider),
            _ => panic!("Provider not implemented for {:?}", self),
        }
    }

    pub fn detect() -> Option<Self> {
        let managers = [
            PackageManager::Apt,
            PackageManager::Dnf,
            PackageManager::Pacman,
            PackageManager::Brew,
            PackageManager::Snap,
            PackageManager::Flatpak,
            PackageManager::Bun,
            PackageManager::Npm,
            PackageManager::Cargo,
            PackageManager::Go,
            PackageManager::Pip,
        ];

        for manager in managers {
            if manager.get_provider().is_available() {
                return Some(manager);
            }
        }

        None
    }
}

/// Helper function to check if a command exists
pub fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}