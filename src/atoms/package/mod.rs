use dhd_macros::typescript_enum;
use std::process::Command;
use std::str::FromStr;

pub mod apt;
pub mod brew;
pub mod bun;
pub mod cargo;
pub mod dnf;
pub mod flatpak;
pub mod github;
pub mod go;
pub mod npm;
pub mod pacman;
pub mod pip;
pub mod snap;
pub mod uv;

#[typescript_enum]
pub enum PackageManager {
    Apt,
    Brew,
    Bun,
    Cargo,
    Dnf,
    Flatpak,
    GitHub,
    Npm,
    Pacman,
    Snap,
    Go,
    Yum,
    Zypper,
    Pip,
    Gem,
    Nix,
    Uv,
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

impl FromStr for PackageManager {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "apt" => Ok(PackageManager::Apt),
            "brew" => Ok(PackageManager::Brew),
            "bun" => Ok(PackageManager::Bun),
            "cargo" => Ok(PackageManager::Cargo),
            "dnf" => Ok(PackageManager::Dnf),
            "flatpak" => Ok(PackageManager::Flatpak),
            "github" => Ok(PackageManager::GitHub),
            "npm" => Ok(PackageManager::Npm),
            "pacman" => Ok(PackageManager::Pacman),
            "snap" => Ok(PackageManager::Snap),
            "go" => Ok(PackageManager::Go),
            "yum" => Ok(PackageManager::Yum),
            "zypper" => Ok(PackageManager::Zypper),
            "pip" => Ok(PackageManager::Pip),
            "gem" => Ok(PackageManager::Gem),
            "nix" => Ok(PackageManager::Nix),
            "uv" => Ok(PackageManager::Uv),
            _ => Err(format!("Unknown package manager: {}", s)),
        }
    }
}

impl PackageManager {
    pub fn get_provider(&self) -> Box<dyn PackageProvider> {
        match self {
            PackageManager::Apt => Box::new(apt::AptProvider),
            PackageManager::Brew => Box::new(brew::BrewProvider),
            PackageManager::Bun => Box::new(bun::BunProvider),
            PackageManager::Cargo => Box::new(cargo::CargoProvider),
            PackageManager::Dnf => Box::new(dnf::DnfProvider),
            PackageManager::Flatpak => Box::new(flatpak::FlatpakProvider),
            PackageManager::GitHub => Box::new(github::GitHubProvider),
            PackageManager::Npm => Box::new(npm::NpmProvider),
            PackageManager::Pacman => Box::new(pacman::PacmanProvider),
            PackageManager::Snap => Box::new(snap::SnapProvider),
            PackageManager::Go => Box::new(go::GoProvider),
            PackageManager::Pip => Box::new(pip::PipProvider),
            PackageManager::Uv => Box::new(uv::UvProvider),
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
            PackageManager::Uv,
        ];

        managers
            .into_iter()
            .find(|manager| manager.get_provider().is_available())
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
