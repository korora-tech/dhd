use crate::{Atom, DhdError, Result};
use std::process::Command;

#[derive(Debug, Clone)]
pub enum PackageManager {
    // System package managers
    Apt,
    Brew,
    Pacman,
    Dnf,
    Yum,
    Zypper,
    Nix,
    // Language package managers
    Npm,
    Yarn,
    Pnpm,
    Bun,
    Deno,
    Pip,
    Pipx,
    Gem,
    Cargo,
    Go,
    Composer,
}

impl PackageManager {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "apt" => Some(Self::Apt),
            "brew" => Some(Self::Brew),
            "pacman" => Some(Self::Pacman),
            "dnf" => Some(Self::Dnf),
            "yum" => Some(Self::Yum),
            "zypper" => Some(Self::Zypper),
            "nix" => Some(Self::Nix),
            "npm" => Some(Self::Npm),
            "yarn" => Some(Self::Yarn),
            "pnpm" => Some(Self::Pnpm),
            "bun" => Some(Self::Bun),
            "deno" => Some(Self::Deno),
            "pip" => Some(Self::Pip),
            "pipx" => Some(Self::Pipx),
            "gem" => Some(Self::Gem),
            "cargo" => Some(Self::Cargo),
            "go" => Some(Self::Go),
            "composer" => Some(Self::Composer),
            _ => None,
        }
    }

    fn command(&self) -> &'static str {
        match self {
            Self::Apt => "apt",
            Self::Brew => "brew",
            Self::Pacman => "pacman",
            Self::Dnf => "dnf",
            Self::Yum => "yum",
            Self::Zypper => "zypper",
            Self::Nix => "nix",
            Self::Npm => "npm",
            Self::Yarn => "yarn",
            Self::Pnpm => "pnpm",
            Self::Bun => "bun",
            Self::Deno => "deno",
            Self::Pip => "pip",
            Self::Pipx => "pipx",
            Self::Gem => "gem",
            Self::Cargo => "cargo",
            Self::Go => "go",
            Self::Composer => "composer",
        }
    }

    fn install_args(&self) -> Vec<&'static str> {
        match self {
            Self::Apt => vec!["install", "-y"],
            Self::Brew => vec!["install"],
            Self::Pacman => vec!["-S", "--noconfirm"],
            Self::Dnf => vec!["install", "-y"],
            Self::Yum => vec!["install", "-y"],
            Self::Zypper => vec!["install", "-y"],
            Self::Nix => vec!["profile", "install"],
            Self::Npm => vec!["install", "-g"],
            Self::Yarn => vec!["global", "add"],
            Self::Pnpm => vec!["add", "-g"],
            Self::Bun => vec!["add", "-g"],
            Self::Deno => vec!["install"],
            Self::Pip => vec!["install"],
            Self::Pipx => vec!["install"],
            Self::Gem => vec!["install"],
            Self::Cargo => vec!["install"],
            Self::Go => vec!["install"],
            Self::Composer => vec!["global", "require"],
        }
    }

    fn check_args(&self) -> Option<Vec<&'static str>> {
        match self {
            Self::Apt => Some(vec!["list", "--installed"]),
            Self::Brew => Some(vec!["list"]),
            Self::Pacman => Some(vec!["-Q"]),
            Self::Dnf => Some(vec!["list", "installed"]),
            Self::Yum => Some(vec!["list", "installed"]),
            Self::Zypper => Some(vec!["search", "--installed-only"]),
            Self::Nix => None, // Complex to check
            Self::Npm => Some(vec!["list", "-g", "--depth=0"]),
            Self::Yarn => Some(vec!["global", "list"]),
            Self::Pnpm => Some(vec!["list", "-g", "--depth=0"]),
            Self::Bun => Some(vec!["pm", "ls", "-g"]),
            Self::Deno => None, // Use which command for deno installed binaries
            Self::Pip => Some(vec!["show"]),
            Self::Pipx => Some(vec!["list"]),
            Self::Gem => Some(vec!["list"]),
            Self::Cargo => None, // Use which command instead
            Self::Go => None,    // Complex to check
            Self::Composer => Some(vec!["global", "show"]),
        }
    }

    fn needs_sudo(&self) -> bool {
        matches!(
            self,
            Self::Apt | Self::Dnf | Self::Yum | Self::Zypper | Self::Pacman
        )
    }

    fn detect_system_manager() -> Option<Self> {
        // Try to detect the system package manager
        if Command::new("apt").arg("--version").output().is_ok() {
            return Some(Self::Apt);
        }
        if Command::new("brew").arg("--version").output().is_ok() {
            return Some(Self::Brew);
        }
        if Command::new("pacman").arg("--version").output().is_ok() {
            return Some(Self::Pacman);
        }
        if Command::new("dnf").arg("--version").output().is_ok() {
            return Some(Self::Dnf);
        }
        if Command::new("yum").arg("--version").output().is_ok() {
            return Some(Self::Yum);
        }
        if Command::new("zypper").arg("--version").output().is_ok() {
            return Some(Self::Zypper);
        }
        if Command::new("nix").arg("--version").output().is_ok() {
            return Some(Self::Nix);
        }
        None
    }
}

pub struct PackageInstall {
    packages: Vec<String>,
    manager: Option<PackageManager>,
}

impl PackageInstall {
    pub fn new(packages: Vec<String>, manager: Option<String>) -> Result<Self> {
        let manager = manager
            .map(|m| {
                PackageManager::from_str(&m).ok_or_else(|| {
                    DhdError::AtomExecution(format!("Unknown package manager: {}", m))
                })
            })
            .transpose()?;

        Ok(Self { packages, manager })
    }

    fn get_manager(&self) -> Result<PackageManager> {
        self.manager
            .clone()
            .or_else(PackageManager::detect_system_manager)
            .ok_or_else(|| {
                DhdError::AtomExecution("Could not determine package manager".to_string())
            })
    }

    fn is_package_installed(&self, package: &str, manager: &PackageManager) -> bool {
        // For cargo and deno, use which to check if binary exists
        if matches!(manager, PackageManager::Cargo | PackageManager::Deno) {
            return Command::new("which")
                .arg(package)
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false);
        }

        // For other managers, use their check commands
        if let Some(check_args) = manager.check_args() {
            let mut cmd = Command::new(manager.command());
            for arg in &check_args {
                cmd.arg(arg);
            }
            cmd.arg(package);

            cmd.output()
                .map(|output| output.status.success())
                .unwrap_or(false)
        } else {
            // If no check command, assume not installed
            false
        }
    }
}

impl Atom for PackageInstall {
    fn check(&self) -> Result<bool> {
        if self.packages.is_empty() {
            return Ok(false);
        }

        let manager = self.get_manager()?;

        // Check if any package is not installed
        for package in &self.packages {
            if !self.is_package_installed(package, &manager) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn execute(&self) -> Result<()> {
        if self.packages.is_empty() {
            return Ok(());
        }

        let manager = self.get_manager()?;

        let mut cmd = if manager.needs_sudo() {
            let mut cmd = Command::new("sudo");
            cmd.arg(manager.command());
            cmd
        } else {
            Command::new(manager.command())
        };

        // Add install arguments
        for arg in manager.install_args() {
            cmd.arg(arg);
        }

        // Add packages
        for package in &self.packages {
            cmd.arg(package);
        }

        tracing::info!(
            "Installing packages with {}: {:?}",
            manager.command(),
            self.packages
        );

        let output = cmd.output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(DhdError::AtomExecution(format!(
                "Failed to install packages: {}",
                stderr
            )));
        }

        Ok(())
    }

    fn describe(&self) -> String {
        let manager_str = self
            .manager
            .as_ref()
            .map(|m| format!(" using {}", m.command()))
            .unwrap_or_else(|| " using system package manager".to_string());

        format!(
            "Install packages{}: {}",
            manager_str,
            self.packages.join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_manager_from_str() {
        assert!(matches!(
            PackageManager::from_str("apt"),
            Some(PackageManager::Apt)
        ));
        assert!(matches!(
            PackageManager::from_str("cargo"),
            Some(PackageManager::Cargo)
        ));
        assert!(PackageManager::from_str("unknown").is_none());
    }

    #[test]
    fn test_package_install_new() {
        let install = PackageInstall::new(vec!["neovim".to_string()], None).unwrap();
        assert_eq!(install.packages, vec!["neovim"]);
        assert!(install.manager.is_none());

        let install =
            PackageInstall::new(vec!["neovim".to_string()], Some("apt".to_string())).unwrap();
        assert!(matches!(install.manager, Some(PackageManager::Apt)));
    }

    #[test]
    fn test_package_install_new_invalid_manager() {
        let result = PackageInstall::new(vec!["neovim".to_string()], Some("invalid".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_package_install_describe() {
        let install =
            PackageInstall::new(vec!["neovim".to_string(), "git".to_string()], None).unwrap();
        assert_eq!(
            install.describe(),
            "Install packages using system package manager: neovim, git"
        );

        let install =
            PackageInstall::new(vec!["neovim".to_string()], Some("apt".to_string())).unwrap();
        assert_eq!(install.describe(), "Install packages using apt: neovim");
    }

    #[test]
    fn test_needs_sudo() {
        assert!(PackageManager::Apt.needs_sudo());
        assert!(!PackageManager::Brew.needs_sudo());
        assert!(!PackageManager::Cargo.needs_sudo());
    }

    #[test]
    fn test_install_args() {
        assert_eq!(PackageManager::Apt.install_args(), vec!["install", "-y"]);
        assert_eq!(PackageManager::Cargo.install_args(), vec!["install"]);
        assert_eq!(PackageManager::Npm.install_args(), vec!["install", "-g"]);
    }
}
